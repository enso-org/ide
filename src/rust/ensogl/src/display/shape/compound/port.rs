//! This module defines the shapes required for drawing node ports.

use crate::data::color::*;
use crate::display::shape::*;
use crate::prelude::*;
use crate::display::shape::primitive::system::ShapeSystem;
use crate::display::symbol::geometry::Sprite;
use crate::display::world::World;
use crate::math::topology::unit::AngleOps;
use crate::math::topology::unit::{Angle, Degrees};
use crate::display::object::Node;
use crate::display;
use crate::display::object::Object;
use nalgebra as na;


// ===========================
// === Port Specification ===
// ===========================

/// Indicates whether a port is incoming or outgoing.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Direction {
    /// Indicates port facing towards the center of its inner circle.
    Inwards,
    /// Indicates port facing away from the center of its inner circle.
    Outwards,
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
    use crate::display::shape::compound::port::{Specification, Direction};
    use crate::display::shape::*;
    use crate::display::shape::primitive::def::modifier::immutable::*;
    use crate::display::shape::primitive::def::primitive::*;
    use crate::prelude::*;
    use crate::display::shape::primitive::def::class::ShapeOps;
    use crate::math::circle_segment::CircleSegment;
    use crate::math::topology::unit::AngleOps;
    use crate::math::topology::unit::Distance;
    use crate::math::topology::unit::PixelDistance;
    use crate::math::topology::unit::Pixels;
    use nalgebra as na;

    /// Construct a port according to the given `PortSpecification`.
    #[allow(clippy::new_ret_no_self)]
    pub fn make_shape(spec:Specification) -> (AnyShape, f32) {
        match spec.direction{
            Direction::Inwards  => new_port_inwards(spec),
            Direction::Outwards => new_port_outwards(spec),
        }
    }


// fn new_port_shape(spec:PortSpecification) -> AnyShape{
//     let inner_radius = spec.inner_radius;
//
//     let segment = CircleSegment::new(inner_radius,spec.width.radians());
//
//
//     let tri_height = spec.height;
//     let tri_base = segment.chord_length();
//     let tri_angle = Angle::from(FRAC_2_PI);
//     let triangle = triangle::Triangle::from_sas(tri_height,tri_base,tri_angle);
//     let angle = triangle.beta;
//
//     let plane_angle  = PlaneAngle(Angle::new((2.0 * PI) - (2.0 * angle.value)));
//     let plane_angle = Rotation(plane_angle, 180.0.deg().radians());
//
//     let tri_top =  inner_radius + spec.height;
//     let offset: Vector2<Distance<Pixels>> =  Vector2::new(Distance::new(0.0),Distance::new(tri_top));
//     let plane_angle = Translate(plane_angle,offset);
//
//     let circle_inner = Circle(spec.inner_radius.px());
//
//     let triangle_rounded = Difference(plane_angle,circle_inner.clone()).fill(spec.color);
//
//     let debug_circle = circle_inner.fill(Srgba::new(0.1,0.1,0.1,0.5));
//
//     (triangle_rounded).into()
// }

    /// Construct an outwards facing port.
    fn new_port_outwards(spec:Specification) -> (AnyShape, f32) {
        debug_assert_eq!(spec.direction, Direction::Outwards);

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

        (triangle_rounded.into(), 0.0)
    }

    /// Construct an inwards facing port.
    fn new_port_inwards(spec:Specification) -> (AnyShape, f32) {
        debug_assert_eq!(spec.direction, Direction::Inwards);

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

        (triangle_rounded.into(), 0.0)

    }

}

//
// struct BoundingBox{
//     top: f32,
//     left: f32,
//     bottom: f32,
//     right: f32,
// }
//
// struct ShapeDescription{
//     pub shape: AnyShape,
//     pub pivot: Vector2<f32>,
//     pub bbox: BoundingBox,
//
// }
// =================
// === Port Node ===
// =================

#[derive(Debug)]
/// Dummy struct for port construction.
pub struct Port {
    spec         : Specification,
    shape        : AnyShape,
    shape_system : ShapeSystem,
    sprite       : Sprite,
    sprite_node  : Node,
}

impl Port {

    pub fn new(spec:Specification, world:&World,parent:&Node) -> Self{
        let (shape, _dh) = shape::make_shape(spec);
        let shape_system = ShapeSystem::new(world,&shape);
        let sprite = shape_system.new_instance();
        sprite.size().set(na::Vector2::new(75.0,75.0));

        let sprite_node = Node::new(Logger::new("node_sprite"));
        parent.add_child(&sprite_node);

        let port = Port{spec,shape,shape_system,sprite,sprite_node};
        port.update_sprite_position_orientation();
        port
    }

    pub fn update(&self) {
        self.shape_system.display_object().update();
        self.update_sprite_position_orientation();
    }

    /// Modifies the rotation of the sprite.
    pub fn mod_specification<F:FnOnce(&mut Specification)>(&mut self, f:F) {
        f(&mut self.spec);
        self.update();
    }

    pub fn update_sprite_position_orientation(&self) {
        let translation_vector = na::Vector3::new(0.0,self.spec.inner_radius,0.0);
        let rotation_vector = -na::Vector3::new(0.0,0.0,self.spec.location.rad().value);
        let rotation = na::Rotation3::new(rotation_vector.clone());
        let translation = rotation * translation_vector;

        self.sprite_node.set_position(translation);
        self.sprite.set_position(dbg!(self.sprite_node.global_position()));
        self.sprite.set_rotation(rotation_vector);
    }

}

impl Into<display::object::Node> for &Port {
    fn into(self) -> display::object::Node {
        (&self.shape_system).into()
    }
}
