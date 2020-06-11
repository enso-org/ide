mod events;

use crate::prelude::*;

use crate::animation::physics;
use crate::control::callback;
use crate::display::camera::Camera2d;
use crate::display::object::traits::*;
use crate::display::Scene;
use crate::system::web::dom;
use crate::system::web;
use events::NavigatorEvents;
use events::PanEvent;
use events::ZoomEvent;



// =================
// === Navigator ===
// =================

/// Navigator enables camera navigation with mouse interactions.
#[derive(Debug)]
pub struct Navigator {
    _events         : NavigatorEvents,
    simulator       : physics::inertia::DynSimulator<Vector3>,
    resize_callback : callback::Handle
}

impl Navigator {
    pub fn new(scene:&Scene, camera:&Camera2d) -> Self {
        let dom                    = scene.dom.root.clone_ref();
        let zoom_speed             = 10.0;
        let min_zoom               = 10.0;
        let max_zoom               = 10000.0;
        let scaled_down_zoom_speed = zoom_speed / 1000.0;
        let (simulator,resize_callback,_events) = Self::start_navigator_events
            (&dom.into(),camera,min_zoom,max_zoom,scaled_down_zoom_speed);
        Self {simulator,_events,resize_callback}
    }

    fn create_simulator(camera:&Camera2d) -> physics::inertia::DynSimulator<Vector3> {
        let camera_ref = camera.clone_ref();
        let update     = Box::new(move |p:Vector3| camera_ref.set_position(p));
        let simulator  = physics::inertia::DynSimulator::new(update);
        simulator.set_value(camera.position());
        simulator.set_target_value(camera.position());
        simulator
    }

    fn start_navigator_events
    ( dom        : &dom::WithKnownShape<web::EventTarget>
    , camera     : &Camera2d
    , min_zoom   : f32
    , max_zoom   : f32
    , zoom_speed : f32
    ) -> (physics::inertia::DynSimulator<Vector3>,callback::Handle,NavigatorEvents) {
        let simulator        = Self::create_simulator(&camera);
        let panning_callback = enclose!((dom,camera,mut simulator) move |pan: PanEvent| {
            let fovy_slope                  = camera.half_fovy_slope();
            let distance                    = camera.position().z;
            let distance_to_show_full_ui    = dom.shape().height / 2.0 / fovy_slope;
            let movement_scale_for_distance = distance / distance_to_show_full_ui;

            // FIXME: Adding - here as panning was accidentally inverted by some recent changes.
            //        Issue tracked by wdanilo and notdanilo.
            let dx   = - pan.movement.x * movement_scale_for_distance;
            let dy   = - pan.movement.y * movement_scale_for_distance;
            let diff = Vector3::new(dx,dy,0.0);
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

        let zoom_callback = enclose!((dom,camera,simulator) move |zoom:ZoomEvent| {
                let point       = zoom.focus;
                let normalized  = normalize_point2(point,dom.shape().into());
                let normalized  = normalized_to_range2(normalized, -1.0, 1.0);
                let half_height = 1.0;

                // Scale X and Y to compensate aspect and fov.
                let x              = -normalized.x * camera.screen().aspect();
                let y              = -normalized.y;
                let z              = half_height / camera.half_fovy_slope();
                let direction      = Vector3(x,y,z).normalize();
                let mut position   = simulator.target_value();
                let min_zoom       = camera.clipping().near + min_zoom;
                let zoom_amount    = zoom.amount * position.z;
                let direction      = direction   * zoom_amount;
                let max_zoom_limit = max_zoom - position.z;
                let min_zoom_limit = min_zoom - position.z;
                let too_far        = direction.z > max_zoom_limit;
                let too_close      = direction.z < min_zoom_limit;
                let zoom_factor    = if too_far   { max_zoom_limit / direction.z }
                                else if too_close { min_zoom_limit / direction.z }
                                else              { 1.0 };
                position          += direction * zoom_factor;
                simulator.set_target_value(position);
        });
        (simulator,resize_callback, NavigatorEvents::new(&dom, panning_callback, zoom_callback, zoom_speed))
    }
}



// =============
// === Utils ===
// =============

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
