//! This module defines the shapes and logic required for drawing node connections.
use crate::prelude::*;

use crate::frp;

use ensogl::data::color::*;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Scene;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::ShapeRegistry;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::def::class::ShapeOps;
use ensogl::display::symbol::geometry::Sprite;
use ensogl::display;
use ensogl::gui::component::ShapeViewDefinition;
use ensogl::gui::component;
use ensogl::math::geometry::triangle::Triangle;
use ensogl::display::layout::alignment;


// =========================
// === Connection Shapes ===
// =========================

mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (start_x:f32,start_y:f32,end_x:f32,end_y:f32,thickness:f32) {

            let height = &start_y - &end_y;
            let width  = &start_x - &end_x;

            let line = Line(1.0);

            let angle_inner = Var::<f32>::from(90_f32.to_radians());
            let triangle    = Triangle::<Var<f32>>::from_sides_and_angle(height,width,angle_inner);
            let angle       = Var::<Angle<Radians>>::from(triangle.angle_a().clone());
            let line        = line.rotate(angle);

            let y_mid = (start_y + end_y) * 0.5;
            let line = line.translate_y(Var::<Distance<Pixels>>::from(y_mid));

            let line_color   = Srgba::new(0.22,0.83,0.54,1.0);
            let line_colored = line.fill(line_color);
            line_colored.into()
        }
    }
}



// ==============
// === Events ===
// ==============

/// Connection events.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Events {
    pub network    : frp::Network,
    pub select     : frp::Source,
    pub deselect   : frp::Source,
}



// ==================
// === Connection ===
// ==================

/// Shape view for connection.
#[derive(Debug,Default,Clone,Copy)]
pub struct ConnectionView {}
impl ShapeViewDefinition for ConnectionView {
    type Shape = shape::Shape;
    fn new(shape:&Self::Shape, _scene:&Scene,shape_registry:&ShapeRegistry) -> Self {
        // FIXME this is only for debuggingg
        shape.start_x.set(-100.0);
        shape.start_y.set(-100.0);
        shape.end_x.set(200.0);
        shape.end_y.set(200.0);

        let bbox = Vector2::new(200.0, 200.0);
        shape.sprite.size().set(bbox);

        let shape_system = shape_registry.shape_system(PhantomData::<shape::Shape>);
        shape_system.shape_system.set_alignment(alignment::HorizontalAlignment::Center, alignment::VerticalAlignment::Center);

        Self {}
    }
}

#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
#[allow(missing_docs)]
pub struct Connection {
    pub data : Rc<ConnectionData>
}


/// Internal data of `Connection`.
#[derive(Debug,Clone)]
#[allow(missing_docs)]
pub struct ConnectionData {
    /// Start of the connection.
    start       : Cell<Vector3<f32>>,
    /// Start of the connection.
    end         : Cell<Vector3<f32>>,
    /// Stroke thickness of the connection.
    stroke_thickness      : Cell<f32>,

    pub logger : Logger,
    pub events : Events,
    pub view   : Rc<component::ShapeView<ConnectionView>>,
    pub label  : frp::Source<String>,
}

impl Connection {

    /// Constructor.
    pub fn new() -> Self {
        frp::new_network! { connection_network
            def label    = source::<String> ();
            def select   = source::<()>     ();
            def deselect = source::<()>     ();
        }
        let network      = connection_network;
        let start        = Cell::new(Vector3::zero());
        let end          = Cell::new(Vector3::zero());
        let stroke_thickness = Cell::new(15.0);
        let logger       = Logger::new("connection");
        let view         = Rc::new(component::ShapeView::new(&logger));
        let events       = Events{network,select,deselect};
        let data         = ConnectionData{start,end,label,events,view,stroke_thickness,logger};
        let data         = Rc::new(data);
        Self {data} . init()
    }

    fn init(self) -> Self {
        self
    }

    /// Set the connections origin.
    pub fn set_start(&self, position:Vector3<f32>) {
        self.data.start.set(position);
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.start_x.set(position.x);
            t.shape.start_y.set(position.y);
        }
        self.update_sprite();
    }

    /// Set the position where the connections ends.
    pub fn set_end(&self, position:Vector3<f32>) {
        self.data.end.set(position);
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.end_x.set(position.x);
            t.shape.end_y.set(position.y);
        }
        self.update_sprite();
    }

    fn center(&self) -> Vector3<f32> {
        (self.data.end.get() + self.data.start.get()) * 0.5
    }

    fn extent(&self) -> Vector3<f32> {
        self.data.end.get() - self.data.start.get()
    }

    fn update_sprite(&self) {
        // let center = self.center();
        // self.data.view.display_object.set_position(Vector3::new(center.x, center.y, 0.0));
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.sprite.size().set(self.extent().xy());
        }
    }
}


impl Default for Connection {
    fn default() -> Self {
        Self::new()
    }
}

impl display::Object for Connection {
    fn display_object(&self) -> &display::object::Instance {
        &self.data.view.display_object
    }
}

impl AsRef<Connection> for Connection {
    fn as_ref(&self) -> &Self {
        self
    }
}
