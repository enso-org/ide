mod events;

use crate::prelude::*;

use crate::animation::physics;
use crate::control::callback;
use crate::display::camera::Camera2d;
use crate::display::object::traits::*;
use crate::display::Scene;

use events::NavigatorEvents;
use events::PanEvent;
use events::ZoomEvent;



// ======================
// === NavigatorModel ===
// ======================

/// Navigator enables camera navigation with mouse interactions.
#[derive(Debug)]
pub struct NavigatorModel {
    _events         : NavigatorEvents,
    simulator       : physics::inertia::DynSimulator<Vector3>,
    resize_callback : callback::Handle,
    zoom_speed      : SharedSwitch<f32>,
    pan_speed       : SharedSwitch<f32>,
    /// Indicates whether events handled the navigator should be stopped from propagating further
    /// after being handled by the Navigator.
    disable_events  : Rc<Cell<bool>>,
}

impl NavigatorModel {
    pub fn new(scene:&Scene, camera:&Camera2d) -> Self {
        let zoom_speed             = Rc::new(Cell::new(Switch::On(10.0/1000.0)));
        let pan_speed              = Rc::new(Cell::new(Switch::On(1.0)));
        let min_zoom               = 10.0;
        let max_zoom               = 10000.0;
        let disable_events         = Rc::new(Cell::new(true));
        let (simulator,resize_callback,_events) = Self::start_navigator_events
            (scene,camera,min_zoom,max_zoom,Rc::clone(&zoom_speed),Rc::clone(&pan_speed),
             Rc::clone(&disable_events));
        Self {_events,simulator,resize_callback,zoom_speed,pan_speed,disable_events}
    }

    fn create_simulator(camera:&Camera2d) -> physics::inertia::DynSimulator<Vector3> {
        let camera_ref = camera.clone_ref();
        let on_step    = Box::new(move |p:Vector3| camera_ref.set_position(p));
        let simulator  = physics::inertia::DynSimulator::new(on_step,(),());
        // FIXME[WD]: This one is emitting camera position in next frame, which is not intended.
        //            Should be fixed when reworking navigator to use FRP events.
        simulator.set_value(camera.position());
        simulator.set_target_value(camera.position());
        simulator
    }

    fn start_navigator_events
    ( scene          : &Scene
    , camera         : &Camera2d
    , min_zoom       : f32
    , max_zoom       : f32
    , zoom_speed     : SharedSwitch<f32>
    , pan_speed      : SharedSwitch<f32>
    , disable_events : Rc<Cell<bool>>
    ) -> (physics::inertia::DynSimulator<Vector3>,callback::Handle,NavigatorEvents) {
        let simulator        = Self::create_simulator(camera);
        let panning_callback = enclose!((scene,camera,mut simulator,pan_speed) move |pan: PanEvent| {
            let fovy_slope                  = camera.half_fovy_slope();
            let distance                    = camera.position().z;
            let distance_to_show_full_ui    = scene.shape().value().height / 2.0 / fovy_slope;
            let pan_speed                   = pan_speed.get().into_on().unwrap_or(0.0);
            let movement_scale_for_distance = distance / distance_to_show_full_ui;
            let diff = pan_speed * Vector3::new(pan.movement.x,pan.movement.y,0.0)*movement_scale_for_distance;
            simulator.update_target_value(|p| p - diff);
        });

        let resize_callback = camera.add_screen_update_callback(
            enclose!((mut simulator,camera) move |_:&Vector2<f32>| {
                let position = camera.position();
                simulator.set_value(position);
                simulator.set_target_value(position);
                simulator.set_velocity(default());
            })
        );

        let zoom_callback = enclose!((scene,camera,simulator) move |zoom:ZoomEvent| {
            let half_screen_size = Vector2::from(scene.shape().value()) / 2.0;
            // We consider the focus point relative to the center of the screen.
            let focus_from_center = zoom.focus - half_screen_size;
            // We scale the focus point, such that y=1 stands for the upper edge and y=-1 for the
            // lower edge of the screen.
            let normalized_focus = focus_from_center / half_screen_size.y;
            // The focus point projected to an imagined plane that is one unit in front of the
            // camera.
            let focus_at_dist_1 = normalized_focus * camera.half_fovy_slope();
            // The direction in which we zoom. We negate x and y, because this vector points behind
            // the camera: When we get a positive scroll input, we want to move the camera
            // backwards, away from the mouse cursor.
            let direction = Vector3(-focus_at_dist_1.x,-focus_at_dist_1.y,1.0);

            let current_position = simulator.target_value();
            // We need to take the exponent because we do multiply `zoom_factor` onto the z
            // coordinate rather than adding it. Taking the exponent makes sure that zooming behaves
            // consistently for large and for small scroll steps. More infos can be found here:
            // https://github.com/enso-org/ide/issues/1613
            let zoom_factor      = zoom.amount.exp2();
            let new_z            = current_position.z * zoom_factor;
            let new_z            = new_z.max(camera.clipping().near + min_zoom).min(max_zoom);
            let zoom_distance    = new_z - current_position.z;
            let zoom_delta       = direction * zoom_distance;
            let new_position     = current_position + zoom_delta;

            simulator.set_target_value(new_position);
        });
        (simulator,resize_callback, NavigatorEvents::new(&scene.mouse.mouse_manager,
                                                         panning_callback,zoom_callback,
                                                         zoom_speed,pan_speed,disable_events))
    }

    pub fn enable(&self) {
        self.pan_speed.update(|switch| switch.switched(true));
        self.zoom_speed.update(|switch| switch.switched(true));
        self.disable_events.set(true);
    }

    pub fn disable(&self) {
        self.pan_speed.update(|switch| switch.switched(false));
        self.zoom_speed.update(|switch| switch.switched(false));
        self.disable_events.set(false);
    }
}



// =================
// === Navigator ===
// =================

/// Navigator enables camera navigation with mouse interactions.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Navigator {
    #[shrinkwrap(main_field)]
    model : Rc<NavigatorModel>,
}

impl Navigator {
    pub fn new(scene:&Scene, camera:&Camera2d) -> Self {
        let model = Rc::new(NavigatorModel::new(scene,camera));
        Navigator{model}
    }

}



// =============
// === Utils ===
// =============

type SharedSwitch<T> = Rc<Cell<Switch<T>>>;

/// Normalize a `point` in (0..dimension.x, 0..dimension.y) to (0..1, 0..1).
fn normalize_point2
(point:Vector2<f32>, dimension:Vector2<f32>) -> Vector2<f32> {
    Vector2::new(point.x / dimension.x, point.y / dimension.y)
}

/// Transforms a `point` normalized in (0..1, 0..1) to (a..b,a..b).
fn normalized_to_range2(point:Vector2<f32>, a:f32, b:f32) -> Vector2<f32> {
    let width = b - a;
    Vector2::new(point.x * width + a, point.y * width + a)
}
