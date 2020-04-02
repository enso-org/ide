//! This module defines the shapes required for drawing node ports.

use ensogl::data::color::*;
use ensogl::display::shape::*;
use ensogl::prelude::*;
use ensogl::display::shape::primitive::system::ShapeSystem;
use ensogl::display::symbol::geometry::Sprite;
use ensogl::display::world::World;
use ensogl::math::topology::unit::AngleOps;
use ensogl::math::topology::unit::{Angle, Degrees};
use ensogl::display::object::Node;
use ensogl::display;
use ensogl::display::object::Object;
use nalgebra as na;
use ensogl::display::object::ObjectOps;
use ensogl::display::Scene;
use ensogl::gui::component;
use ensogl::display::scene::ShapeRegistry;

// ===========================
// === Port Specification ===
// ===========================

/// Indicates whether a port is incoming or outgoing.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Direction {
    /// Indicates port facing towards the center of its inner circle.
    In,
    /// Indicates port facing away from the center of its inner circle.
    Out,
}


/// Defines the properties of a port shape and can then
/// be used to build the port shape.
///
/// Ports are constructed around an inner circle, and thus
/// most measurements are in degrees, which are measured around
/// a inner circle, that is defined by the `inner_radius`.
#[derive(Clone,Copy,Debug)]
pub struct Specification{
    /// Height of the port.
    pub height      : f32,
    /// Width of the port in degrees.
    pub width       : Angle<Degrees>,
    /// Radius of the inner circle that the port is constructed around.
    pub inner_radius: f32,
    /// Direction the port is facing.
    pub direction   : Direction,
    /// Location of the port along the inner circle.
    pub location    : Angle<Degrees>,
    /// Fill colour of the port.
    pub color       : Srgb<f32>,
}

// ==================
// === Port Shape ===
// ==================

mod shape{
    use super::*;
    use ensogl::display::shape::*;
    use ensogl::display::shape::primitive::def::modifier::immutable::*;
    use ensogl::display::shape::primitive::def::primitive::*;
    use ensogl::prelude::*;
    use ensogl::display::shape::primitive::def::class::ShapeOps;
    use ensogl::math::geometry::circle::circle_segment::CircleSegment;
    use ensogl::math::topology::unit::AngleOps;
    use ensogl::math::topology::unit::Distance;
    use ensogl::math::topology::unit::PixelDistance;
    use ensogl::math::topology::unit::Pixels;
    use nalgebra as na;

    /// Construct a port according to the given `PortSpecification`.
    #[allow(clippy::new_ret_no_self)]
    pub fn make_shape(spec:Specification) -> AnyShape {
        match spec.direction{
            // TODO consider unifying shape creation
            Direction::In => new_port_inwards(spec),
            Direction::Out => new_port_outwards(spec),
        }
    }

    /// Construct an outwards facing port.
    fn new_port_outwards(spec:Specification) -> AnyShape {
        debug_assert_eq!(spec.direction, Direction::Out);

        let inner_radius = spec.inner_radius;
        let segment = CircleSegment::new(inner_radius,spec.width.radians());

        // Create the triangle (pointing up)
        let tri_base   = segment.sagitta();
        let tri_height = spec.height + segment.sagitta();
        let tri_width  = segment.chord_length();

        let triangle = Triangle(tri_width,tri_height);

        let circle_inner = Circle(spec.inner_radius.px());
        let circle_offset: na::Vector2<Distance<Pixels>> =  na::Vector2::new(Distance::new(0.0),Distance::new(segment.sagitta()-spec.inner_radius));
        let circle_inner = Translate(circle_inner,circle_offset);

        let triangle_rounded = Difference(triangle,circle_inner.clone()).fill(spec.color);
        let tri_offset: Vector2<Distance<Pixels>> =  Vector2::new(Distance::new(0.0),Distance::new(-tri_base));
        let triangle_rounded = Translate(triangle_rounded,tri_offset);

        triangle_rounded.into()
    }

    /// Construct an inwards facing port.
    fn new_port_inwards(spec:Specification) -> AnyShape {
        debug_assert_eq!(spec.direction, Direction::In);

        let outer_radius = spec.inner_radius + spec.height;
        let segment = CircleSegment::new(outer_radius,spec.width.radians());

        // Create the triangle (pointing up)
        let tri_height =  spec.height;
        let tri_width  = segment.chord_length() * ((outer_radius + segment.sagitta()) / outer_radius);

        // Point the triangle down
        let triangle = Triangle(tri_width, tri_height);
        let triangle = Rotation(triangle, 180.0.deg().radians());

        // Move the triangle to its base position and rotate it to its final destination.
        let offfset: Vector2<Distance<Pixels>> =  Vector2::new(Distance::new(0.0),Distance::new(tri_height));
        let triangle = Translate(triangle,offfset);

        let circle_outer = Circle(outer_radius.px());
        let circle_offset: Vector2<Distance<Pixels>> =  Vector2::new(Distance::new(0.0),Distance::new(tri_height-outer_radius));
        let circle_outer = Translate(circle_outer,circle_offset);
        let triangle_rounded = Intersection(triangle,circle_outer.clone()).fill(spec.color);

        triangle_rounded.into()

    }


    /// Canvas node shape definition.

        ensogl::define_shape_system! {
        () {
        // TODO take spec or spec values as `Var<_>` parameters
        let node_radius = 60.0 ;
        let port_height = 30.0;

        let port_spec = Specification{
            height: port_height,
            width: Angle::from(25.0),
            inner_radius: node_radius,
            direction: Direction::Out,
            location: 90.0_f32.deg(),
            color: Srgb::new(0.26, 0.69, 0.99),
        };

          make_shape(port_spec)
        }
    }
}



// =================
// === Port Node ===
// =================

/// Shape view for Port.
#[derive(Debug,Clone,Copy)]
pub struct PortView {}
impl component::ShapeViewDefinition for PortView {
    type Shape = shape::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene, _shape_registry:&ShapeRegistry) -> Self {
        shape.sprite.size().set(Vector2::new(200.0,200.0));
        Self {}
    }
}


/// Weak version of `Port`.
#[derive(Clone,CloneRef,Debug)]
pub struct WeakPort {
    data : Weak<PortData>
}

#[derive(Debug)]
/// Internal data of `Port`
pub struct PortData {
    pub view     : component::ShapeView<PortView>,
}

/// Port definition.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct Port {
    spec         : Specification,
    #[shrinkwrap(main_field)]
    data         : Rc<PortData>,
}

impl Port {

    pub fn new(spec:Specification) -> Self{
        let logger = Logger::new("node");
        let view   = component::ShapeView::new(&logger);

        let port_data = PortData{view};
        let data   = Rc::new(port_data);

        Self{spec,data}.init()
    }

    fn init(mut self) -> Self{
        self.update();
        self
    }

    fn update(&mut self) {
        // TODO this should probably happen somewhere else.
        self.update_sprite_position_orientation();
        self.data.view.display_object.update();
    }

    /// Modifies the rotation of the sprite.
    pub fn mod_specification<F:FnOnce(&mut Specification)>(&mut self, f:F) {
        f(&mut self.spec);
        self.update();
        // TODO remove
        println!("PORT UPDATE!");
    }

    fn update_sprite_position_orientation(&mut self) {
        let translation_vector = na::Vector3::new(0.0,self.spec.inner_radius,0.0);
        let rotation_vector = -na::Vector3::new(0.0,0.0,self.spec.location.rad().value);
        let rotation = na::Rotation3::new(rotation_vector.clone());
        let translation = rotation * translation_vector;

        let node = &self.data.view.display_object;
        node.set_position(translation);
        // self.sprite.set_position(dbg!(self.sprite_node.global_position()));
        node.set_rotation(rotation_vector);
        // TODO also update shape
    }

}


impl Drop for PortData{
    fn drop(&mut self){
        // TODO remove
        println!("PORT  DROP!");
    }
}

// impl StrongRef for Port {
//     type WeakRef = WeakPort;
//     fn downgrade(&self) -> WeakPort {
//         WeakPort {data:Rc::downgrade(&self.data)}
//     }
// }

// impl WeakRef for WeakPort {
//     type StrongRef = Port;
//     fn upgrade(&self) -> Option<Port> {
//         self.data.upgrade().map(|data| Port{data})
//     }
// }

impl<'t> From<&'t Port> for &'t display::object::Node {
    fn from(t:&'t Port) -> Self {
        &t.data.view.display_object
    }
}