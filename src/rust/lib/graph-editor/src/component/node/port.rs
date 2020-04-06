//! This module defines the shapes and logic required for drawing node ports.

use crate::component::node::WeakNode;

use ensogl::data::color::*;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Scene;
use ensogl::display::object::Object;
use ensogl::display::object::Node;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::ShapeRegistry;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::def::class::ShapeOps;
use ensogl::display::symbol::geometry::Sprite;
use ensogl::display;
use ensogl::gui::component::ShapeViewDefinition;
use ensogl::gui::component;
use ensogl::math::geometry::circle::circle_segment::CircleSegment;
use ensogl::math::topology::unit::AngleOps;
use ensogl::math::topology::unit::Distance;
use ensogl::math::topology::unit::Pixels;
use ensogl::math::topology::unit::{Angle,Degrees};
use ensogl::prelude::*;
use nalgebra as na;



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

    /// Construct an inwards facing port.
    fn new_port_inwards(spec:SpecificationVar) -> AnyShape {
        let outer_radius     : Var<f32> = (spec.inner_radius + spec.height.clone()).into();
        let segment_width_rad: Var<f32> = spec.width.clone().into();
        let segment_radius   : Var<f32> = outer_radius.clone();
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
        let circle_outer    = Circle(circle_radius);

        let circle_offset_y: Var<Distance<Pixels>> = (&tri_height - &outer_radius).into();
        let circle_outer    = circle_outer.translate_y(circle_offset_y);

        let triangle_rounded = Intersection(triangle,circle_outer);
        let triangle_rounded = triangle_rounded.fill(Srgb::new(0.26, 0.69, 0.99));

        // Center triangle
        let center_offset: Var<Distance<Pixels>> = ((&tri_height + segment.sagitta()) * Var::from(0.5)).into();
        let triangle_rounded = triangle_rounded.translate_y(-center_offset);

        triangle_rounded.into()
    }

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

    /// Construct an outwards facing port.
    fn new_port_outwards(spec:SpecificationVar) -> AnyShape {
        let inner_radius : Var<f32> = spec.inner_radius.into();
        let height       : Var<f32> = spec.height.clone().into();

        let segment_width_rad: Var<f32>       = spec.width.clone().into();
        let segment_radius   : Var<f32>       = inner_radius.clone();
        let segment : CircleSegment<Var<f32>> = CircleSegment::new(segment_radius,segment_width_rad);

        let tri_base             = segment.sagitta();
        let tri_height: Var<f32> = &height + &tri_base;
        let tri_width            = segment.chord_length();

        // TODO[mm] consider replace triangle with a `Plane().cut_angle`
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

        // Center triangle
        let center_offset: Var<Distance<Pixels>> = (&tri_height * Var::from(0.5)).into();
        let triangle_rounded = triangle_rounded.translate_y(-center_offset);

        triangle_rounded.into()
    }

    ensogl::define_shape_system! {
        (height:f32,width:f32,inner_radius:f32) {
            // TODO[mm] take spec or spec values as `Var<_>` parameters
            let port_spec_val = SpecificationVar {
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
const DEFAULT_HEIGHT : f32 = 30.0;


// TODO consider unifying the ShapeViews.
/// Shape view for Input Port.
#[derive(Debug,Clone,Copy)]
pub struct InputPortView {}
impl ShapeViewDefinition for InputPortView {
    type Shape = shape_in::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene, _shape_registry:&ShapeRegistry) -> Self {
        // FIXME This is an approximation and needs to be computed exactly to avoid clipping in edge cases.
        let bbox = Vector2::new(1.5 * DEFAULT_HEIGHT,1.5 * DEFAULT_HEIGHT);
        shape.sprite.size().set(bbox);
        Self {}
    }
}

/// Shape view for Output Port.
#[derive(Debug,Clone,Copy)]
pub struct OutputPortView {}
impl ShapeViewDefinition for OutputPortView {
    type Shape = shape_out::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene, _shape_registry:&ShapeRegistry) -> Self {
        // FIXME This is an approximation and needs to be computed exactly to avoid clipping in edge cases.
        let bbox = Vector2::new(1.5 * DEFAULT_HEIGHT,1.5 * DEFAULT_HEIGHT);
        shape.sprite.size().set(bbox);
        Self {}
    }
}

/// Port definition. Can be parametrised to be either
/// an InputPort or OutputPort.
#[derive(Debug,Clone)]
#[allow(missing_docs)]
pub struct Port<T:ShapeViewDefinition> {
        spec     : Specification,
    pub view     : Rc<component::ShapeView<T>>
}

impl<T:ShapeViewDefinition> Port<T> {

    /// Constructor.
    pub fn new(spec:Specification) -> Self {
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
        self.view.display_object.update();
    }

    /// Update the position of the sprite according to the Port specification.
    /// The position is given along a circle, thus the position and rotation of the sprite
    /// are tied together, so the Port always point in the right direction.
    fn update_sprite(&mut self) {
        let translation_vector = na::Vector3::new(0.0,self.spec.inner_radius,0.0);
        let rotation_vector = -na::Vector3::new(0.0,0.0,self.spec.location.rad().value);
        let rotation = na::Rotation3::new(rotation_vector);
        let translation = rotation * translation_vector;

        let node = &self.view.display_object;
        node.set_position(translation);
        node.set_rotation(rotation_vector);
    }

}

/// A port facing towards the center of its inner circle.
pub type InputPort = Port<InputPortView>;

/// A port facing away from the center of its inner circle.
pub type OutputPort = Port<OutputPortView>;


impl<'t, T:ShapeViewDefinition> From<&'t Port<T>> for &'t Node {
    fn from(t:&'t Port<T>) -> Self {
        &t.view.display_object
    }
}

impl<T:ShapeViewDefinition> Drop for Port<T> {
    fn drop(&mut self) {
        println!("DROP")
    }
}



// ====================
// === Port Manager ===
// ====================

/// Handles creation and layouting of ports around a node.
/// TODO implement the layouting
#[derive(Debug,Default)]
pub struct PortManager {
    parent       : RefCell<Option<WeakNode>>,
    input_ports  : RefCell<Vec<InputPort>>,
    output_ports : RefCell<Vec<OutputPort>>,
}

impl PortManager{

    /// Set the parent node of the created `Ports`.
    ///
    /// Needs to be set after creation for circular dependecy reasons.
    pub fn set_parent(&self, parent:WeakNode) {
        self.parent.set(parent);
    }

    fn add_child_to_parent<T:Object>(&self, child:&T) {
        if let Some(weak_node) = self.parent.borrow().as_ref() {
            if let Some(node) = weak_node.upgrade() {
                node.add_child(child);
            }
        }
    }

    /// Create a new InpuTtPort.
    pub fn create_input_port(&self) {
        // TODO layouting for multiple nodes
        let node_radius = 68.0 ;
        let port_height = 30.0;

        let port_spec = Specification {
            height       : port_height,
            width        : Angle::from(25.0),
            inner_radius : node_radius,
            location     : 90.0_f32.deg(),
            color        : Srgb::new(51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0 ),
        };

        let port = InputPort::new(port_spec);
        self.add_child_to_parent(&port);
        self.input_ports.borrow_mut().push(port);
        self.update_ports();
    }

    /// Create a new OutputPort.
    pub fn create_output_port(&self) {
        // TODO layouting for multiple nodes
        let node_radius = 68.0 ;
        let port_height = 30.0;

        let port_spec = Specification {
            height       : port_height,
            width        : Angle::from(25.0),
            inner_radius : node_radius,
            location     : 270.0_f32.deg(),
            color        : Srgb::new(51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0 ),
        };

        let port = OutputPort::new(port_spec);
        self.add_child_to_parent(&port);
        self.output_ports.borrow_mut().push(port);
        self.update_ports();
    }

    /// Update the shapes of all ports.
    fn update_ports(& self) {
        for port in self.input_ports.borrow_mut().iter_mut() {
            if let Some(t) = port.view.data.borrow().as_ref() {
                    t.shape.width.set(port.spec.width.value.to_radians());
                    t.shape.inner_radius.set(port.spec.inner_radius);
                    t.shape.height.set(port.spec.height);
            }
            port.update()
        }
        for port in self.output_ports.borrow_mut().iter_mut() {
            if let Some(t) = port.view.data.borrow().as_ref() {
                t.shape.width.set(port.spec.width.value.to_radians());
                t.shape.inner_radius.set(port.spec.inner_radius);
                t.shape.height.set(port.spec.height);
            }
            port.update()
        }
    }
}
