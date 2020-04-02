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


#[derive(Clone,Debug)]
pub struct SpecificationVar{
    /// Height of the port.
    pub height      : Var<Distance<Pixels>>,
    /// Width of the port in degrees.
    pub width       : Var<Angle<Radians>>,
    /// Radius of the inner circle that the port is constructed around.
    pub inner_radius: Var<Distance<Pixels>>,
    /// Location of the port along the inner circle.
    pub location    : Var<Angle<Radians>>,
}



// ==================
// === Port Shape ===
// ==================

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
        // TODO cut down on clone usage

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
        // TODO consider replace with a `Plane().cut_angle`
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
        () {
        // TODO take spec or spec values as `Var<_>` parameters

        let port_spec_val = SpecificationVar{
            height       : Var::from(15.px()),
            width        : Var::from(Angle::<Degrees>::from(25.0).rad()),
            inner_radius : Var::from(48.px()),
            location     : Var::from(Angle::<Degrees>::from(45.0).rad()),
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
        // TODO cut down on clone usage

        let inner_radius : Var<f32> = spec.inner_radius.into();
        let height       : Var<f32> = spec.height.clone().into();

        let segment_width_rad: Var<f32>       = spec.width.clone().into();
        let segment_radius   : Var<f32>       = inner_radius.clone().into();
        let segment : CircleSegment<Var<f32>> = CircleSegment::new(segment_radius,segment_width_rad);

        let tri_base             = segment.sagitta();
        let tri_height: Var<f32> =  &height + &tri_base;
        let tri_width            = segment.chord_length();

        // TODO consider replace triangle with a `Plane().cut_angle`
        // But avoid visual artifacts at the other end of the circle.
        // let section = Plane().cut_angle(&spec.width);
        // let section = section.rotate(180.0.deg().radians());
        // let section = section.translate_y(tri_offset);

        let triangle = Triangle(&tri_width, &tri_height);

        let circle_radius: Var<Distance<Pixels>> = inner_radius.clone().into();
        let circle_inner    = Circle((circle_radius));

        let circle_offset_y: Var<Distance<Pixels>> = (&tri_base-&inner_radius).into();
        let circle_inner    = circle_inner.translate_y((circle_offset_y));

        let triangle_rounded = Difference(triangle,circle_inner);
        let triangle_rounded = triangle_rounded.fill(Srgb::new(0.26, 0.69, 0.99));

        let tri_offset: Var<Distance<Pixels>> = (-&tri_base).into();
        let triangle_rounded = triangle_rounded.translate_y(tri_offset);

        triangle_rounded.into()
    }

    /// Canvas node shape definition.
    ensogl::define_shape_system! {
        () {
        // TODO take spec or spec values as `Var<_>` parameters

        let port_spec_val = SpecificationVar{
            height       : Var::from(15.px()),
            width        : Var::from(Angle::<Degrees>::from(25.0).rad()),
            inner_radius : Var::from(48.px()),
            location     : Var::from(Angle::<Degrees>::from(45.0).rad()),
        };

          new_port_outwards(port_spec_val)
        }
    }
}


// =================
// === Port Node ===
// =================

/// Shape view for Input Port.
#[derive(Debug,Clone,Copy)]
pub struct InputPortView {}
impl component::ShapeViewDefinition for InputPortView {
    type Shape = shape_in::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene, _shape_registry:&ShapeRegistry) -> Self {
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
        shape.sprite.size().set(Vector2::new(200.0,200.0));
        Self {}
    }
}


// /// Weak version of `Port`.
// #[derive(Clone,CloneRef,Debug)]
// pub struct WeakPort {
//     data : Weak<PortData>
// }

#[derive(Debug)]
/// Internal data of `Port`
pub struct PortData {
    pub view     : component::ShapeView<InputPortView>,
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
        // TODO remove
        println!("PORT UPDATE!");
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
//
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
