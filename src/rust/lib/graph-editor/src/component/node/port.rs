//! This module defines the shapes and logic required for drawing node ports.

use crate::component::node::WeakNode;

use ensogl::data::color::*;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Scene;
use ensogl::display::object::Object;
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
use nalgebra;



// ===========================
// === Port Specification ===
// ===========================

/// Indicates whether a port is inwards facing  or outwards facing.
#[allow(missing_docs)]
#[derive(Clone,Copy,Debug)]
pub enum Direction{
    In,
    Out,
}

/// Defines the properties of a port shape and can then
/// be used to build the port shape.
///
/// Ports are constructed around an inner circle, and thus
/// most measurements are in degrees, which are measured around
/// a inner circle that is defined by the `inner_radius`.
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
    /// Indicates whether this port is facing inwards or outwards.
    pub direction   : Direction,
}

impl Default for Specification {
    fn default() -> Self {
        Specification {
            height       : 20.0,
            width        : Angle::from(15.0),
            inner_radius : 70.0,
            location     : Angle::from(0.0),
            color        : Srgb::new(0.26, 0.69, 0.99),
            direction    : Direction::Out,
        }
    }
}



// ====================
// === Port Shapes ===
// ====================


mod shape {
    use super::*;

    /// Construct an inwards facing port.
    ///
    /// The port consists of an angle embedded between two rings. The corner of the angle can
    /// either face towards the center of thr ring of away from it. If it faces towards the center,
    /// the side facing the outer ring, have a bulging shape on that side. If the angle faces away
    /// from the center, the angle gets limited by the inner ring and shows a concave "hole" on
    /// that side.
    ///
    /// Illustrations (not to scale).
    /// ---------------
    /// Inwards facing port                           | Outwards facing port
    ///                                               |
    ///                                               |
    ///       *-------------------- inner radius      |          *  ---------------- outer radius
    ///     *    *                |                   |        *   *               |
    ///   *        *              |  height           |      *       *             |
    ///  *          *             |                   |     *    *    * ------------ inner radius
    ///   *        *              |                   |    * *       * *      |
    ///      *  *------------------ outer radius      |    *            *     - this is some extra
    ///                                                                        space that depends
    ///  \----------|                                      \------------|       on the radius
    ///     width                                              width
    ///
    /// The shape description sets up both shapes and then applies only one of the variants:
    /// either it limits of the inward facing angle with the outer ring or it cuts of the
    /// angle with the inner ring.
    ///
    fn new_port(height:Var<f32>,width:Var<f32>,inner_radius:Var<f32>,_is_inwards:Var<f32>) -> AnyShape {

        let zoom_factor                  = Var::<f32>::from("1.0 / input_zoom");
        let height                       = &height * &zoom_factor;
        let outer_radius      : Var<f32> = &inner_radius + &height;
        let segment_width_rad : Var<f32> = &width * &zoom_factor;

        // This describes the segment between the angle and the outer ring.
        let segment_outer_radius  = outer_radius.clone();
        let segment_outer        = CircleSegment::new(
            segment_outer_radius,segment_width_rad.clone()
        );

        // This describes the segment between the angle and the inner ring.
        let segment_inner_radius = inner_radius.clone();
        let segment_inner = CircleSegment::new(segment_inner_radius,segment_width_rad);

        // The triangle needs to be high enough for it to have room for the extra shape.
        let shape_height = height + segment_outer.sagitta();
        let shape_width = segment_outer.chord_length() * ((&outer_radius + segment_outer.sagitta()) / &outer_radius);

        // Position the triangle facing down with its base+extension at the zero mark.
        let base_shape = Triangle(&shape_width, &shape_height);
        let base_shape   = base_shape.rotate(180.0.deg().radians());

        // After rotating the shape down, we need to move it up by its height again.
        let base_shape_offset = Var::<Distance<Pixels>>::from(shape_height.clone());
        let base_shape        = base_shape.translate_y(base_shape_offset);

        // TODO[mm] consider replace with a `Plane().cut_angle`
        // Set up the angle and rotate it so it points downwards like a "V".
        // FIXME this should be an angle that is computed dynamically
        // let section = Plane().cut_angle(Var::from(45_f32.to_radians()));
        // let section = section.rotate(180.0.deg().radians());
        // After rotating it needs to be aligned so it's tip is at (0,0) again.
        // let section = section.translate_y(tri_baseline_offset);

        // `circle_outer_radius_scale` toggles whether the circle will have any effect on the
        // shape. This circle will be used with an `Union`, thus a large enough radius will
        // negate its effect.
        let circle_outer_radius_scale = Var::<f32>::from("1.0 + (input_is_inwards * 999999.9)");
        let circle_outer_radius       = circle_outer_radius_scale * &outer_radius;
        let circle_outer_radius       = Var::<Distance<Pixels>>::from(circle_outer_radius);
        let circle_outer              = Circle(circle_outer_radius);

        // `circle_inner_radius_scale` toggles whether the circle will have any effect on the
        // shape. This circle will be used with an `Difference`, thus a zero radius will negate
        // its effect.
        let circle_inner_radius_scale = Var::<f32>::from("input_is_inwards * 1.0");
        let circle_inner_radius       = circle_inner_radius_scale * &inner_radius;
        let circle_inner_radius       = Var::<Distance<Pixels>>::from(circle_inner_radius);
        let circle_inner              = Circle(circle_inner_radius);

        // We don't want to move around the angle so we position the two circles relative to the
        // angle.

        let circle_outer_offset_y = Var::<Distance<Pixels>>::from(
            &shape_height
             - segment_outer.sagitta()
             - &outer_radius
        );
        let circle_outer          = circle_outer.translate_y(circle_outer_offset_y);

        let circle_inner_offset_y = Var::<Distance<Pixels>>::from(
            &shape_height
             - segment_outer.sagitta()
             - segment_inner.sagitta()
             + &inner_radius
        );
        let circle_inner          = circle_inner.translate_y(circle_inner_offset_y);

        // Now we shape the angle by applying the circles.Note that only one of them will have an
        // effect, based on the radius that we modified set through `input_is_inwards`.
        let sculpted_shape = base_shape;
        let sculpted_shape = Intersection(sculpted_shape,circle_outer);
        let sculpted_shape = Difference(sculpted_shape,circle_inner);
        let sculpted_shape = sculpted_shape.fill(Srgb::new(0.26, 0.69, 0.99));

        // The angle should be centered on (0,0) to make it easier to rotate and minimise the
        // required canvas.
        let center_offset    = Var::<Distance<Pixels>>::from(&shape_height * Var::from(0.5));
        let sculpted_shape   = sculpted_shape.translate_y(-center_offset);

        // This is a conditional rotation that allows the port to either point inwards or outwards.
        let rotation_angle = Var::<Angle<Radians>>::from("Radians(input_is_inwards * 3.1415926538)");
        let sculpted_shape = sculpted_shape.rotate(rotation_angle);

        sculpted_shape.into()
    }

    ensogl::define_shape_system! {
        (height:f32,width:f32,inner_radius:f32,is_inwards:f32) {
            /// NOTE: `is_inwards` should only be 0.0 or 1.0.
            new_port(height,width,inner_radius,is_inwards)
        }
    }

    impl Shape{
        /// Set the shape parameters derived from the `Specification`.
        pub fn update_from_spec(&self,spec:&Specification){
            self.height.set(spec.height);
            self.inner_radius.set(spec.inner_radius);
            self.width.set(spec.width.value.to_radians());
            match &spec.direction{
                Direction::In  => self.is_inwards.set(1.0),
                Direction::Out => self.is_inwards.set(0.0),
            };
        }
    }
}



// ============
// === Port ===
// ============

/// Shape view for Input Port.
#[derive(Debug,Clone,Copy)]
pub struct InputPortView {}
impl ShapeViewDefinition for InputPortView {
    type Shape = shape::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene,_shape_registry:&ShapeRegistry) -> Self {
        let spec  = Specification::default();
        shape.update_from_spec(&spec);
        shape.is_inwards.set(1.0);

        let bbox = Vector2::new(2.0 * shape.height.get(), 2.0 * shape.height.get());
        shape.sprite.size().set(bbox);

        Self {}
    }
}

/// Shape view for Input Port.
#[derive(Debug,Clone,Copy)]
pub struct OutputPortView {}
impl ShapeViewDefinition for OutputPortView {
    type Shape = shape::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene,_shape_registry:&ShapeRegistry) -> Self {
        let spec  = Specification::default();
        shape.update_from_spec(&spec);
        shape.is_inwards.set(0.0);

        let bbox = Vector2::new(2.0 * shape.height.get(), 2.0 * shape.height.get());
        shape.sprite.size().set(bbox);

        Self {}
    }
}

/// Port definition.
#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct Port<T:ShapeViewDefinition+Clone> {
    pub data : Rc<PortData<T>>
}

/// Type that represents an input port.
pub type InputPort = Port<InputPortView>;

/// Type that represents an output port.
pub type OutputPort = Port<OutputPortView>;

/// Internal data of `Port.
#[derive(Debug,Clone)]
#[allow(missing_docs)]
pub struct PortData<T:ShapeViewDefinition+Clone> {
        spec : RefCell<Specification>,
    pub view : Rc<component::ShapeView<T>>
}

impl<T:ShapeViewDefinition<Shape=shape::Shape>+Clone> Port<T> {

    /// Constructor.
    pub fn new(spec:Specification) -> Self {
        let logger = Logger::new("node");
        let view   = Rc::new(component::ShapeView::new(&logger));
        let spec = RefCell::new(spec);
        let data = PortData{spec,view};
        Self{data: Rc::new(data)}.init()
    }

    fn init(self) -> Self {
        self.update();
        self
    }

    /// Modifies the port specification.
    pub fn mod_specification<F:FnOnce(&mut Specification)>(&mut self, f:F) {
        f(&mut self.data.spec.borrow_mut());
        self.update()
    }

    /// Update the shape parameters and sprite location with values from our current specification.
    fn update(&self) {
        self.update_shape();
        self.update_sprite();
    }

    /// Update the position of the sprite according to the Port specification.
    /// The position is given along a circle, thus the position and rotation of the sprite
    /// are tied together, so the Port always point in the right direction.
    fn update_sprite(&self) {
        let spec = self.data.spec.borrow();
        let translation_vector = nalgebra::Vector3::new(0.0,spec.inner_radius,0.0);
        let rotation_vector    = -nalgebra::Vector3::new(0.0,0.0,spec.location.rad().value);
        let rotation           = nalgebra::Rotation3::new(rotation_vector);
        let translation        = rotation * translation_vector;

        let node = &self.data.view.display_object;
        node.set_position(translation);
        node.set_rotation(rotation_vector);
    }

    /// Update the shape parameters with values from our `Specification`.
    fn update_shape(&self){
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            let spec = self.data.spec.borrow();
            t.shape.update_from_spec(&spec);
        }
    }
}

impl<'t,T:ShapeViewDefinition+Clone> From<&'t Port<T>> for &'t display::object::Node {
    fn from(t:&'t Port<T>) -> Self {
        &t.data.view.display_object
    }
}



// ====================
// === Port Manager ===
// ====================

/// Handles creation and layouting of ports around a node.
/// TODO implement the layouting
#[derive(Debug,Default)]
pub struct PortManager {
    /// The node that all ports will be placed around.
    parent_node  : RefCell<Option<WeakNode>>,
    input_ports  : RefCell<Vec<InputPort>>,
    output_ports : RefCell<Vec<OutputPort>>,
}

impl PortManager{

    /// Set the parent node of the created `Ports`.
    ///
    /// Needs to be set after creation for circular dependency reasons.
    pub fn set_parent(&self, parent:WeakNode) {
        self.parent_node.set(parent);
    }

    fn add_child_to_parent<T:Object>(&self, child:&T) {
        if let Some(weak_node) = self.parent_node.borrow().as_ref() {
            if let Some(node) = weak_node.upgrade() {
                node.add_child(child);
            }
        }
    }

    /// Create a new InputPort.
    pub fn create_input_port(&self) {
        // TODO layouting for multiple nodes

        let port_spec = Specification {
            location  : 90.0_f32.deg(),
            direction : Direction::In,
            ..Default::default()
        };

        let port = Port::new(port_spec);
        self.add_child_to_parent(&port);
        self.input_ports.borrow_mut().push(port);
        self.update_port_shapes();
    }

    /// Create a new OutputPort.
    pub fn create_output_port(&self) {
        // TODO layouting for multiple nodes

        let port_spec = Specification {
            location  : 270.0_f32.deg(),
            direction : Direction::Out,
            ..Default::default()
        };

        let port = Port::new(port_spec);
        self.add_child_to_parent(&port);
        self.output_ports.borrow_mut().push(port);
        self.update_port_shapes();
    }

    /// Update the shapes of all ports with the currently set specification values.
    fn update_port_shapes(& self) {
        for port in self.input_ports.borrow_mut().iter_mut() {
            port.update()
        }
        for port in self.output_ports.borrow_mut().iter_mut() {
            port.update()
        }
    }
}
