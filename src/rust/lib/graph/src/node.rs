
use crate::prelude::*;

use crate::HasSprite;

use ensogl::control::callback::CallbackMut1;
use ensogl::data::color::Srgba;
use ensogl::display;
use ensogl::display::traits::*;
use ensogl::display::Sprite;
use ensogl::math::Vector2;
use ensogl::math::Vector3;
use logger::Logger;
use std::any::TypeId;
use enso_prelude::std_reexports::fmt::{Formatter, Error};
use ensogl::animation::physics::inertia::DynInertiaSimulator;
use enso_frp;
use enso_frp as frp;
use enso_frp::frp;
use enso_frp::core::node::class::EventEmitterPoly;
use ensogl::display::{AnyBuffer,Buffer};
use ensogl::data::color::*;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::system::ShapeSystem;
use ensogl::display::world::World;
use ensogl::display::scene::{Scene,Component};



pub mod icons {
    use super::*;

    pub fn history() -> AnyShape {
        let radius_diff    = 0.5.px();
        let corners_radius = 2.0.px();
        let width_diff     = &corners_radius * 3.0;
        let offset         = 2.px();
        let width          = 32.px();
        let height         = 16.px();
        let persp_diff1    = 6.px();

        let width2          = &width  - &width_diff;
        let width3          = &width2 - &width_diff;
        let corners_radius2 = &corners_radius  - &radius_diff;
        let corners_radius3 = &corners_radius2 - &radius_diff;
        let persp_diff2     = &persp_diff1 * 2.0;

        let rect1 = Rect((&width ,&height)).corners_radius(&corners_radius);
        let rect2 = Rect((&width2,&height)).corners_radius(&corners_radius2).translate_y(&persp_diff1);
        let rect3 = Rect((&width3,&height)).corners_radius(&corners_radius3).translate_y(&persp_diff2);

        let rect3 = rect3 - rect2.translate_y(&offset);
        let rect2 = rect2 - rect1.translate_y(&offset);

        let rect1 = rect1.fill(Srgba::new(0.26, 0.69, 0.99, 1.00));
        let rect2 = rect2.fill(Srgba::new(0.26, 0.69, 0.99, 0.6));
        let rect3 = rect3.fill(Srgba::new(0.26, 0.69, 0.99, 0.4));

        let icon = (rect3 + rect2 + rect1).translate_y(-persp_diff2/2.0);
        icon.into()
    }
}

pub fn ring_angle<R,W,A>(inner_radius:R, width:W, angle:A) -> AnyShape
    where R : Into<Var<Distance<Pixels>>>,
          W : Into<Var<Distance<Pixels>>>,
          A : Into<Var<Angle<Radians>>> {
    let inner_radius = inner_radius.into();
    let width        = width.into();
    let angle        = angle.into();

    let angle2  = &angle / 2.0;
    let radius  = &width / 2.0;
    let inner   = Circle(&inner_radius);
    let outer   = Circle(&inner_radius + &width);
    let section = Plane().cut_angle(&angle);
    let corner1 = Circle(&radius).translate_y(inner_radius + radius);
    let corner2 = corner1.rotate(&angle2);
    let corner1 = corner1.rotate(-&angle2);
    let ring    = &outer - &inner;
    let pie     = &ring * &section;
    let out     = &pie + &corner1 + &corner2;
    let out     = out.fill(Srgba::new(0.9,0.9,0.9,1.0));
    out.into()
}

pub fn shape() -> AnyShape {
    let node_radius = 32.0;
    let border_size = 16.0;

    let node = Circle(node_radius.px());
    let node = node.fill(Srgb::new(0.97,0.96,0.95));
    let bg   = Circle((node_radius*2.0).px());
    let bg   = bg.fill(Srgb::new(0.91,0.91,0.90));

    let shadow2 = Circle((node_radius + border_size).px());
    let shadow2_color = LinearGradient::new()
        .add(0.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear())
        .add(1.0,Srgba::new(0.0,0.0,0.0,0.14).into_linear());
    let shadow2_color = SdfSampler::new(shadow2_color).max_distance(border_size).slope(Slope::Exponent(4.0));
    let shadow2       = shadow2.fill(shadow2_color);

    let selection = Circle((node_radius - 1.0).px() + border_size.px() * "input_selection");
    let selection = selection.fill(Srgba::new(0.22,0.83,0.54,1.0));

    let loader_angle : Var<Angle<Radians>> = "Radians(clamp(input_time/2000.0 - 1.0) * 1.99 * PI)".into();
    let loader_angle2 = &loader_angle / 2.0;
    let loader        = ring_angle((node_radius).px(), (border_size).px(), loader_angle);
    let loader        = loader.rotate(loader_angle2);
    let loader        = loader.rotate("Radians(input_time/200.0)");
    let icon          = icons::history();
    let out           = loader + selection + shadow2 + node + icon;
    out.into()
}




impl Component for Node {
    type ComponentSystem = NodeSystem;
}

#[derive(Clone,Debug)]
pub struct NodeSystem {
    pub shape_system     : ShapeSystem,
    pub selection_buffer : Buffer<f32>
}

impl CloneRef for NodeSystem {
    fn clone_ref(&self) -> Self {
        let shape_system     = self.shape_system.clone_ref();
        let selection_buffer = self.selection_buffer.clone_ref();
        Self {shape_system,selection_buffer}
    }
}

impl NodeSystem {
    pub fn new(scene:&Scene) -> Self {
        let shape_system     = ShapeSystem::new(scene,&shape());
        let selection_buffer = shape_system.add_input("selection", 0.0);
        Self {shape_system,selection_buffer}
    }
}


#[derive(Derivative,Clone,Default)]
#[derivative(Debug)]
pub struct NodeRegistry {
    #[derivative(Debug="ignore")]
    pub map : Rc<RefCell<HashMap<usize,Node>>>
}

type EditCallback = Box<dyn Fn(&Node) + 'static>;

// FIXME We should use real event registers here instead.
#[derive(Default)]
pub struct OnEditCallbacks {
    pub label_changed    : Option<EditCallback>,
    pub color_changed    : Option<EditCallback>,
    pub position_changed : Option<EditCallback>,
    pub removed          : Option<EditCallback>,
}

impl Debug for OnEditCallbacks {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("node::OnEditCallback")
    }
}

#[derive(Debug,Default)]
pub struct NodeData {
    label : String,
    color : Srgba, // FIXME what is node color?
}

#[derive(Derivative,Clone)]
#[derivative(Debug)]
pub struct Node {
    logger         : Logger,
    sprite         : Rc<CloneCell<Option<Sprite>>>,
    display_object : display::object::Node,
    data           : Rc<RefCell<NodeData>>,
    // FIXME: Refcells should be as deep as possible. Each callback manager should have internal mut
    // pattern. This way you can register callbacks while running other callbacks.
    callbacks      : Rc<RefCell<OnEditCallbacks>>,
    simulator : DynInertiaSimulator<f32>,
    pub selection : enso_frp::Dynamic<()>,
}

impl CloneRef for Node {}

impl Node {
    pub fn new(registry:&NodeRegistry) -> Self {
        let logger = Logger::new("node");
        let sprite : Rc<CloneCell<Option<Sprite>>> = default();
        let display_object      = display::object::Node::new(&logger);
        let display_object_weak = display_object.downgrade();


        frp! {
            selection           = source::<()>            ();
            selected            = selection.toggle        ();
            selection_animation = source::<f32>           ();
//            debug = selection.map(|t| {println!("SS: {:?}",t);})

        }

        selection_animation.map("animation", enclose!((sprite) move |value| {
            sprite.get().for_each(|sprite| {
                let symbol = &sprite.symbol;
                let id     = sprite.instance_id;
                let any_buffer = symbol.surface().instance_scope().buffer("selection").unwrap();
                match any_buffer {
                    AnyBuffer::VariantIdentityForf32(buffer) => {
                        buffer.at((*id as usize).into()).set(*value);
                    }
                    _ => todo!()
                }
            })
        }));

        let simulator = DynInertiaSimulator::<f32>::new(Box::new(move |t| {
            selection_animation.event.emit(t);
        }));

        selected.map("selection", enclose!((simulator) move |check| {
            let value = if *check { 1.0 } else { 0.0 };
            simulator.set_target_position(value);
        }));


        let data      = default();
        let callbacks = default();
        let display_object2 = display_object.clone_ref();
        let sprite2 = sprite.clone_ref();

        let this = Self {logger,sprite,display_object,data,callbacks,simulator,selection};

        let sprite = sprite2;

        display_object2.set_on_show_with(enclose!((this,registry,sprite) move |scene| {
            let node_system = scene.shapes.get(PhantomData::<Node>).unwrap();
            let new_sprite  = node_system.shape_system.new_instance();
            display_object_weak.upgrade().for_each(|t| t.add_child(&new_sprite));
            new_sprite.size().set(Vector2::new(200.0,200.0));
            registry.map.borrow_mut().insert(*new_sprite.instance_id,this.clone());
            sprite.set(Some(new_sprite));
        }));


        display_object2.set_on_hide_with(enclose!((registry,sprite) move |_| {
            sprite.get().for_each(|sprite| {
                registry.map.borrow_mut().remove(&*sprite.instance_id);
            });
            sprite.set(None);
        }));


        this
    }

    pub fn set_on_edit_callbacks(&self, callbacks: OnEditCallbacks) {
        *self.callbacks.borrow_mut() = callbacks
    }
}

//impl Default for Node {
//    fn default() -> Self {
//        Node::new()
//    }
//}

// === Interface for library users ===

impl Node {
    // FIXME this is bad. It does not cover all position modifiers like `mod_position`. Should be
    // done as transform callback instead.
    pub fn set_position(&self, pos:Vector3<f32>) {
        self.display_object.set_position(pos);
    }

    pub fn set_label(&self, new_label:String) {
        self.data.borrow_mut().label = new_label;
        //TODO[ao] update sprites
    }

    pub fn set_color(&self, new_color:Srgba) {
        self.data.borrow_mut().color = new_color;
        //TODO[ao] update sprites
    }

    pub fn remove_from_graph(&self) {
        todo!()
    }
}


// === Interface for GUI events ===

impl Node {
    // FIXME this is bad. It does not cover all position modifiers like `mod_position`. Should be
    // done as transform callback instead.
    pub fn gui_set_position(&self, pos:Vector3<f32>) {
        self.set_position(pos);
        self.call_edit_callback(&self.callbacks.borrow().position_changed);
    }

    pub fn gui_set_label(&self, new_label:String) {
        self.set_label(new_label);
        //TODO[ao] update sprites
        self.call_edit_callback(&self.callbacks.borrow().label_changed);
    }

    pub fn gui_set_color(&self, new_color:Srgba) {
        self.set_color(new_color);
        //TODO[ao] update sprites
        self.call_edit_callback(&self.callbacks.borrow().color_changed);
    }

    pub fn gui_remove_from_graph(&self) {
        todo!()
    }

    fn call_edit_callback(&self, callback:&Option<EditCallback>) {
        if let Some(callback) = callback {
            callback(self)
        }
    }
}

// === Getters ===

impl Node {
    pub fn label(&self) -> String {
        self.data.borrow().label.clone()
    }

    pub fn color(&self) -> Srgba {
        self.data.borrow().color.clone()
    }
}

impl HasSprite for Node {
    fn set_sprite(&self, sprite:&Sprite) {
        self.sprite.set(Some(sprite.clone()))
    }
}

impl<'t> From<&'t Node> for &'t display::object::Node {
    fn from(node:&'t Node) -> Self {
        &node.display_object
    }
}
