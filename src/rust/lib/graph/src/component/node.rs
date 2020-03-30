
use crate::prelude::*;

use ensogl::control::callback::CallbackMut1;
use ensogl::data::color::Srgba;
use ensogl::display;
use ensogl::display::traits::*;
use ensogl::display::{Sprite, Attribute};
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
use ensogl::display::shape::primitive::system::ShapeSystemDefinition;
use ensogl::display::world::World;
use ensogl::display::scene::{Scene,MouseTarget,ShapeRegistry};
use ensogl::gui::component::Component;


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



// ============
// === Node ===
// ============

pub mod shape {
    use super::*;

    ensogl::shape! {
        (selection:f32) {
            let node_radius = 32.0;
            let border_size = 16.0;

            let node = Circle(node_radius.px());
            let node = node.fill(Srgb::new(0.97,0.96,0.95));
            let bg   = Circle((node_radius*2.0).px());
            let bg   = bg.fill(Srgb::new(0.91,0.91,0.90));

            let shadow       = Circle((node_radius + border_size).px());
            let shadow_color = LinearGradient::new()
                .add(0.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear())
                .add(1.0,Srgba::new(0.0,0.0,0.0,0.14).into_linear());
            let shadow_color = SdfSampler::new(shadow_color).max_distance(border_size).slope(Slope::Exponent(4.0));
            let shadow       = shadow.fill(shadow_color);

            let selection = Circle((node_radius - 1.0).px() + border_size.px() * "input_selection");
            let selection = selection.fill(Srgba::new(0.22,0.83,0.54,1.0));

            let loader_angle : Var<Angle<Radians>> = "Radians(clamp(input_time/2000.0 - 1.0) * 1.99 * PI)".into();
            let loader        = ring_angle((node_radius).px(), (border_size).px(), &loader_angle);
            let loader        = loader.rotate(loader_angle / 2.0);
            let loader        = loader.rotate("Radians(input_time/200.0)");
            let icon          = icons::history();
            let out           = loader + selection + shadow + node + icon;
            out.into()
        }
    }
}

#[derive(Clone,CloneRef,Debug)]
pub struct Events {
    pub mouse_down : frp::Dynamic<()>,
}

#[derive(Clone,CloneRef,Debug)]
pub struct Node {
    pub logger         : Logger,
    pub display_object : display::object::Node,
    pub label          : frp::Dynamic<String>,
    pub events         : Events,
    pub shape          : Rc<RefCell<Option<shape::ShapeDefinition>>>,
}

impl Component for Node {
    fn on_view_cons(&self, _scene:&Scene, shape_registry:&ShapeRegistry) {
        let shape = shape_registry.new_instance::<shape::ShapeDefinition>();
        self.display_object.add_child(&shape);
        shape.sprite.size().set(Vector2::new(200.0,200.0));
        shape_registry.insert_mouse_target(*shape.sprite.instance_id,self.clone_ref());
        *self.shape.borrow_mut() = Some(shape);
    }
}

impl Node {
    pub fn new() -> Self {
        frp! {
            label      = source::<String> ();
            mouse_down = source::<()>     ();
        }

        let logger         = Logger::new("node");
        let display_object = display::object::Node::new(&logger);
        let events         = Events {mouse_down};
        let shape          = default();
        Self {logger,display_object,label,events,shape} . component_init() . init()
    }

    fn init(self) -> Self {
        let mouse_down = &self.events.mouse_down;

        frp! {
            selected            = mouse_down.toggle ();
            selection_animation = source::<f32>     ();
        }

        let shape = &self.shape;
        selection_animation.map("animation", enclose!((shape) move |value| {
            shape.borrow().as_ref().for_each(|t| t.selection.set(*value))
        }));

        let simulator = DynInertiaSimulator::<f32>::new(Box::new(move |t| {
            selection_animation.event.emit(t);
        }));

        selected.map("selection", enclose!((simulator) move |check| {
            let value = if *check { 1.0 } else { 0.0 };
            simulator.set_target_position(value);
        }));
        self
    }

}

impl MouseTarget for Node {
    fn mouse_down(&self) -> Option<&frp::Dynamic<()>> {
        Some(&self.events.mouse_down)
    }
}

impl<'t> From<&'t Node> for &'t display::object::Node {
    fn from(t:&'t Node) -> Self {
        &t.display_object
    }
}
