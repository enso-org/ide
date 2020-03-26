//! This module defines the shapes required for drawing node ports.

use crate::data::color::*;
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
use crate::math::topology::unit::{Angle, Degrees};

use nalgebra::Vector2;



// ===========================
// === Port Specification ===
// ===========================

/// Indicates whether a port is incoming or outgoing.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum PortDirection {
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
pub struct PortSpecification{
    /// Height of the port.
    pub height      : f32,
    /// Width of the port in degrees.
    pub width       : Angle<Degrees>,
    /// Radius of the inner circle that the port is constructed around.
    pub inner_radius: f32,
    /// Direction the port is facing.
    pub direction   : PortDirection,
    /// Location of the port along the inner circle.
    pub location    : Angle<Degrees>,
    /// Fill colour of the port.
    pub color       : Srgb<f32>,
}


// ============
// === Port ===
// =============

#[derive(Clone,Copy,Debug)]
/// Dummy struct for port construction.
pub struct Port;

impl Port{
    /// Construct a port according to the given `PortSpecification`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(spec:PortSpecification) -> AnyShape{
        let show_debug = false;
        match spec.direction{
            PortDirection::Inwards  => Port::new_port_inwards(spec, show_debug),
            PortDirection::Outwards => Port::new_port_outwards(spec, show_debug),

        }
    }

    /// Construct an outwards facing port.
    fn new_port_outwards(spec:PortSpecification, show_debug:bool) -> AnyShape{
        debug_assert_eq!(spec.direction,PortDirection::Outwards);

        let inner_radius = spec.inner_radius;

        let segment = CircleSegment::new(inner_radius,spec.width.radians());

        // Create the triangle (pointing up)
        let tri_base   = inner_radius - segment.sagitta();
        let tri_height = spec.height + segment.sagitta();
        let tri_width  = segment.chord_length();

        let triangle = Triangle(tri_width,tri_height);

        // Move the triangle to its base position and rotate it to its final destination.
        let offfset: Vector2<Distance<Pixels>> =  Vector2::new(Distance::new(0.0),Distance::new(tri_base));
        let triangle = Translate(triangle,offfset);
        let triangle = Rotation(triangle,spec.location.radians());

        let circle_inner = Circle(spec.inner_radius.px());

        let triangle_rounded = Difference(triangle,circle_inner.clone()).fill(spec.color);

        if show_debug{
            let debug_circle = circle_inner.fill(Srgba::new(0.1,0.1,0.1,0.5));
            (triangle_rounded + debug_circle).into()
        } else{
            triangle_rounded.into()
        }

    }

    /// Construct an inwards facing port.
    fn new_port_inwards(spec:PortSpecification, show_debug:bool) -> AnyShape{
        debug_assert_eq!(spec.direction,PortDirection::Inwards);

        let outer_radius = spec.inner_radius + spec.height;

        let segment = CircleSegment::new(outer_radius,spec.width.radians());

        // Create the triangle (pointing up)
        let tri_height =  spec.height;
        let tri_width  = segment.chord_length() * ((outer_radius + segment.sagitta()) / outer_radius);

        // Point the triangle down
        let triangle = Triangle(tri_width, tri_height);
        let triangle = Rotation(triangle, 180.0.deg().radians());

        // Move the triangle to its base position and rotate it to its final destination.
        let tri_base = outer_radius;
        let offfset: Vector2<Distance<Pixels>> =  Vector2::new(Distance::new(0.0),Distance::new(tri_base));
        let triangle = Translate(triangle,offfset);
        let triangle = Rotation(triangle,spec.location.radians());

        let circle_outer = Circle(outer_radius.px());

        let triangle_rounded = Intersection(triangle,circle_outer.clone()).fill(spec.color);

        if show_debug{
            let debug_circle = circle_outer.fill(Srgba::new(0.1,0.1,0.1, 0.5));
            (triangle_rounded + debug_circle).into()
        } else{
            triangle_rounded.into()
        }
    }
}
