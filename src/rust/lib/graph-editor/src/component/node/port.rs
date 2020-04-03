//! This module defines the shapes required for drawing node ports.

use crate::prelude::*;
use ensogl::prelude::*;

use ensogl::data::color::*;
use ensogl::display::Buffer;
use ensogl::display::Scene;
use ensogl::display::object::Node;
use ensogl::display::object::Object;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::ShapeRegistry;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::system::ShapeSystem;
use ensogl::display::symbol::geometry::Sprite;
use ensogl::display::world::World;
use ensogl::display::Attribute;
use ensogl::display;
use ensogl::gui::component;
use ensogl::math::topology::unit::AngleOps;
use ensogl::math::topology::unit::{Angle, Degrees};

use core::f32::consts::PI;
use nalgebra as na;
use ensogl::gui::component::ShapeViewDefinition;


// ===========================
// === Port Specification ===
// ===========================


/// Defines the properties of a port shape and can then
/// be used to build the port shape.
///
/// Ports are constructed around an inner circle, and thus
/// most measurements are in degrees, which are measured around
/// a inner circle, that is defined by the `inner_radius`.
#[derive(Clone,Copy,Debug)]
pub struct Specification {
    /// Height of the port.
    pub height      : f32,
    /// Width of the port in degrees.
    pub width       : Angle<Degrees>,
    /// Radius of the inner circle that the port is constructed around.
    pub inner_radius: f32,
    /// Location of the port along the inner circle.
    pub location    : Angle<Degrees>,
    /// Fill colour of the port.
    /// TODO unused in the shape at the moment
    pub color       : Srgb<f32>,
}


#[derive(Clone,Debug)]
/// Internal helper struct to aggregate parameters.
/// TODO[mm] consider removing.
struct SpecificationVar {
    /// Height of the port.
    pub height      : Var<Distance<Pixels>>,
    /// Width of the port in degrees.
    pub width       : Var<Angle<Radians>>,
    /// Radius of the inner circle that the port is constructed around.
    pub inner_radius: Var<Distance<Pixels>>,
}



// ====================
// === Port Shapes ===
// ====================

mod shape_in{
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


    /// Construct an inwards facing port.
    fn new_port_inwards(spec:SpecificationVar) -> AnyShape {
        // TODO[mm] cut down on clone usage

        let outer_radius     : Var<f32> = (spec.inner_radius + spec.height.clone()).into();
        let segment_width_rad: Var<f32> = spec.width.clone().into();
        let segment_radius   : Var<f32> = outer_radius.clone().into();
        let segment : CircleSegment<Var<f32>> = CircleSegment::new(segment_radius,segment_width_rad);

        // Create the triangle (pointing up)
        let tri_height: Var<f32> =  spec.height.clone().into();
        let tri_width  = segment.chord_length() * ((&outer_radius + segment.sagitta()) / &outer_radius);

        let triangle = Triangle(&tri_width, &tri_height);
        let triangle = triangle.rotate(180.0.deg().radians());
        let tri_offset: Var<Distance<Pixels>> = tri_height.clone().into();
        let triangle = triangle.translate_y(tri_offset);
        // TODO[mm] consider replace with a `Plane().cut_angle`
        // But avoid visual artifacts at the other end of the circle.
        // let section = Plane().cut_angle(&spec.width);
        // let section = section.rotate(180.0.deg().radians());
        // let section = section.translate_y(tri_offset);

        let circle_radius: Var<Distance<Pixels>> = outer_radius.clone().into();
        let circle_outer    = Circle((circle_radius));

        let circle_offset_y: Var<Distance<Pixels>> = (&tri_height - &outer_radius).into();
        let circle_outer    = circle_outer.translate_y((circle_offset_y));

        let triangle_rounded = Intersection(triangle,circle_outer);
        let triangle_rounded = triangle_rounded.fill(Srgb::new(0.26, 0.69, 0.99));

        triangle_rounded.into()
    }

    /// Canvas node shape definition.
    ensogl::define_shape_system! {
        (height:f32,width:f32,inner_radius:f32) {
            // TODO[mm] take spec or spec values as `Var<_>` parameters
            let port_spec_val = SpecificationVar{
                    height       : height.into(),
                    width        : width.into(),
                    inner_radius : inner_radius.into()
            };
            new_port_inwards(port_spec_val)
        }
    }

}


mod shape_out{
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

    /// Construct an outwards facing port.
    fn new_port_outwards(spec:SpecificationVar) -> AnyShape {
        // TODO[mm] cut down on clone usage

        let inner_radius : Var<f32> = spec.inner_radius.into();
        let height       : Var<f32> = spec.height.clone().into();

        let segment_width_rad: Var<f32>       = spec.width.clone().into();
        let segment_radius   : Var<f32>       = inner_radius.clone().into();
        let segment : CircleSegment<Var<f32>> = CircleSegment::new(segment_radius,segment_width_rad);

        let tri_base             = segment.sagitta();
        let tri_height: Var<f32> = &height + &tri_base;
        let tri_width            = segment.chord_length();

        // TODO[mm]] consider replace triangle with a `Plane().cut_angle`
        // But avoid visual artifacts at the other end of the circle.
        // let section = Plane().cut_angle(&spec.width);
        // let section = section.rotate(180.0.deg().radians());
        // let section = section.translate_y(tri_offset);

        let triangle = Triangle(&tri_width, &tri_height);

        let circle_radius: Var<Distance<Pixels>> = inner_radius.clone().into();
        let circle_inner    = Circle(circle_radius);

        let circle_offset_y: Var<Distance<Pixels>> = (&tri_base-&inner_radius).into();
        let circle_inner    = circle_inner.translate_y(circle_offset_y);

        let triangle_rounded = Difference(triangle,circle_inner);
        let triangle_rounded = triangle_rounded.fill(Srgb::new(0.26, 0.69, 0.99));

        let tri_offset: Var<Distance<Pixels>> = (-&tri_base).into();
        let triangle_rounded = triangle_rounded.translate_y(tri_offset);

        triangle_rounded.into()
    }

    /// Canvas node shape definition.
    ensogl::define_shape_system! {
        (height:f32,width:f32,inner_radius:f32) {
            // TODO[mm] take spec or spec values as `Var<_>` parameters
            let port_spec_val = SpecificationVar{
                    height       : height.into(),
                    width        : width.into(),
                    inner_radius : inner_radius.into()
            };
            new_port_outwards(port_spec_val)
        }
    }
}


// =================
// === Port Node ===
// =================

// TODO[mm] remove and use values derived from node instead
const DEFAULT_WIDTH  : f32 = 25.0 * (PI / 180.0);
const DEFAULT_RADIUS : f32 = 60.0;
const DEFAULT_HEIGHT : f32 = 30.0;


/// TODO consider unifying the ShapeViews.
/// Shape view for Input Port.
#[derive(Debug,Clone,Copy)]
pub struct InputPortView {}
impl component::ShapeViewDefinition for InputPortView {
    type Shape = shape_in::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene, _shape_registry:&ShapeRegistry) -> Self {
        shape.width.set(DEFAULT_WIDTH);
        shape.inner_radius.set(DEFAULT_RADIUS);
        shape.height.set(DEFAULT_HEIGHT);

        shape.sprite.size().set(Vector2::new(200.0,200.0));
        Self {}
    }
}

/// Shape view for Output Port.
#[derive(Debug,Clone,Copy)]
pub struct OutputPortView {}
impl component::ShapeViewDefinition for OutputPortView {
    type Shape = shape_out::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene, _shape_registry:&ShapeRegistry) -> Self {
        shape.width.set(DEFAULT_WIDTH);
        shape.inner_radius.set(DEFAULT_RADIUS);
        shape.height.set(DEFAULT_HEIGHT);
        // TODO[mm] minimse port size
        shape.sprite.size().set(Vector2::new(200.0,200.0));
        Self {}
    }
}


/// Port definition. Can be parametrised to be either
/// an InputPort or OutputPort.
#[derive(Debug,Clone)]
pub struct Port<T:ShapeViewDefinition> {
        spec     : Specification,
    pub view     : Rc<component::ShapeView<T>>
}

impl<T:ShapeViewDefinition> Port<T> {

    pub fn new(spec:Specification) -> Self{
        let logger = Logger::new("node");
        let view   = Rc::new(component::ShapeView::<T>::new(&logger));
        Self{spec,view}.init()
    }

    fn init(mut self) -> Self {
        self.update();
        self
    }

    /// Modifies the port specification.
    pub fn mod_specification<F:FnOnce(&mut Specification)>(&mut self, f:F) {
        f(&mut self.spec);
        self.update()
    }

    /// Update the view with our current Specification.
    fn update(&mut self) {
        self.update_sprite();
        self.update_shape();
        self.view.display_object.update();
    }

    /// Update the position of the sprite according to the Port specification.
    /// The position is given along a circle, thus the position and rotation of the sprite
    /// are tied together, so the Port always point in the right direction.
    fn update_sprite(&mut self){
        let translation_vector = na::Vector3::new(0.0,self.spec.inner_radius,0.0);
        let rotation_vector = -na::Vector3::new(0.0,0.0,self.spec.location.rad().value);
        let rotation = na::Rotation3::new(rotation_vector.clone());
        let translation = rotation * translation_vector;

        let node = &self.view.display_object;
        node.set_position(translation);
        node.set_rotation(rotation_vector);
    }

    /// Update the input parameters of the shape with our spec.
    fn update_shape(&mut self){
        // TODO needs to update the shapeview
        // if let Some(t) = self.view.data.borrow().as_ref(){
        //     t.shape.width.set(self.spec.width.value.to_radians());
        //     t.shape.inner_radius.set(self.spec.inner_radius);
        //     t.shape.height.set(self.spec.height);
        // }
    }
}

/// A port facing towards the center of its inner circle.
pub type InputPort = Port<InputPortView>;

/// A port facing away from the center of its inner circle.
pub type OutputPort = Port<OutputPortView>;


impl<'t, T:ShapeViewDefinition> From<&'t Port<T>> for &'t display::object::Node {
    fn from(t:&'t Port<T>) -> Self {
        &t.view.display_object
    }
}
