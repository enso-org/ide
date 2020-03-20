#![feature(associated_type_defaults)]
#![feature(drain_filter)]
#![feature(overlapping_marker_traits)]
#![feature(specialization)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(unboxed_closures)]
#![feature(weak_into_raw)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod node;

use prelude::*;

use ensogl::display;
use ensogl::display::Sprite;
use ensogl::display::shape::*;
use ensogl::display::object::traits::*;
use ensogl::traits::*;
use logger::Logger;
use enso_prelude::std_reexports::fmt::{Formatter, Error};
use nalgebra::Vector2;

pub mod prelude {
    pub use enso_prelude::*;
}

pub use node::Node;

use ensogl::prelude::*;
use ensogl::traits::*;

use ensogl::data::color::*;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::system::ShapeSystem;
use ensogl::display::shape::Var;
use ensogl::display::world::*;
use ensogl::system::web;
use shapely::shared;
use std::any::TypeId;


// =========================
// === Library Utilities ===
// =========================

pub trait HasSprite {
    fn set_sprite(&self, sprite:&Sprite);
}


// TODO[ao] copied from shapes example, consider where to move it.
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

fn ring_angle<R,W,A>(inner_radius:R, width:W, angle:A) -> AnyShape
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
//    let out     = out.fill(Srgba::new(0.22,0.83,0.54,1.0));
//    let out     = out.fill(Srgba::new(0.0,0.0,0.0,0.2));
    let out     = out.fill(Srgba::new(0.9,0.9,0.9,1.0));
    out.into()
}

fn nodes2() -> AnyShape {
    let node_radius = 32.0;
    let border_size = 16.0;
    let node   = Circle(node_radius.px());
//    let border = Circle((node_radius + border_size).px());
    let node   = node.fill(Srgb::new(0.97,0.96,0.95));
//    let node   = node.fill(Srgb::new(0.26,0.69,0.99));
//    let border = border.fill(Srgba::new(0.0,0.0,0.0,0.06));

    let bg   = Circle((node_radius*2.0).px());
    let bg   = bg.fill(Srgb::new(0.91,0.91,0.90));


//    let shadow1 = Circle((node_radius + border_size).px());
//    let shadow1_color = LinearGradient::new()
//        .add(0.0,Srgba::new(0.0,0.0,0.0,0.08).into_linear())
//        .add(1.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear());
//    let shadow1_color = SdfSampler::new(shadow1_color).max_distance(border_size).slope(Slope::InvExponent(5.0));
//    let shadow1       = shadow1.fill(shadow1_color);

    let shadow2 = Circle((node_radius + border_size).px());
    let shadow2_color = LinearGradient::new()
        .add(0.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear())
        .add(1.0,Srgba::new(0.0,0.0,0.0,0.14).into_linear());
//    let shadow2_color = ExponentSampler::new(shadow2_color);
    let shadow2_color = SdfSampler::new(shadow2_color).max_distance(border_size).slope(Slope::Exponent(4.0));
    let shadow2       = shadow2.fill(shadow2_color);


    let loader_angle : Var<Angle<Radians>> = "Radians(clamp(input_time/2000.0 - 1.0) * 1.99 * PI)".into();
    let loader_angle2 = &loader_angle / 2.0;
    let loader        = ring_angle((node_radius).px(), (border_size).px(), loader_angle);
    let loader        = loader.rotate(loader_angle2);
    let loader        = loader.rotate("Radians(input_time/200.0)");

    let icon = icons::history();


    let out = loader + shadow2 + node + icon;
    out.into()
}



/// Registers Node shape.
pub fn register_shapes(world:&World) {
   let node_shape   = nodes2();
   let shape_system = ShapeSystem::new(world,&node_shape);
   world.scene().register_shape(TypeId::of::<Node>(),shape_system.clone());
}



// =============
// === Graph ===
// =============

#[derive(Default)]
pub struct OnEditCallbacks {
    node_added : Option<Box<dyn Fn(&node::Node) + 'static>>,
}

impl Debug for OnEditCallbacks {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("graph::OnEditCallbacks")
    }
}

#[derive(Debug,Default)]
struct GraphData {
    nodes : Vec<node::Node>,
}

#[derive(Debug)]
pub struct Graph {
    data           : Rc<RefCell<GraphData>>,
    display_object : display::object::Node,
    callbacks      : Rc<RefCell<OnEditCallbacks>>,
    logger         : Logger,
}

impl Graph {
    pub fn new(world:&World) -> Self {
        let logger         = Logger::new("graph");
        let data           = default();
        let display_object = display::object::Node::new(&logger);
        let callbacks      = default();
        register_shapes(world);
        Self {data,display_object,callbacks,logger}
    }

    pub fn set_on_edit_callbacks(&self, callbacks: OnEditCallbacks) {
        *self.callbacks.borrow_mut() = callbacks
    }
}

impl<'a> From<&'a Graph> for &'a display::object::Node {
    fn from(graph: &'a Graph) -> Self {
        &graph.display_object
    }
}


// === Interface for library users ===

impl Graph {
    pub fn add_node(&self, new_node:node::Node) {
        self.display_object.add_child(&new_node);
        self.data.borrow_mut().nodes.push(new_node);
    }

    pub fn clear_graph(&self) {
        let mut data = self.data.borrow_mut();
        for node in &data.nodes {
            self.display_object.remove_child(node);
        }
        data.nodes.clear();
    }
}


// === Interface for GUI events ===

impl Graph {
    pub fn gui_add_node(&self, new_node:node::Node) {
        self.gui_add_node(new_node.clone());
        if let Some(callback) = &self.callbacks.borrow().node_added {
            callback(&new_node)
        }
    }
}