//! This module defines the shapes and logic required for drawing node connections.
use crate::prelude::*;

use crate::frp;
use crate::component::node::port::{WeakPort, OutputPortView, InputPortView, OutputPort, InputPort};

use enso_prelude::StrongRef;
use ensogl::data::color::*;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Scene;
use ensogl::display::layout::alignment;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::ShapeRegistry;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::def::class::ShapeOps;
use ensogl::display::symbol::geometry::Sprite;
use ensogl::display;
use ensogl::gui::component::ShapeViewDefinition;
use ensogl::gui::component;
use ensogl::math::geometry::triangle::Triangle;



// ========================
// === Connection Shape ===
// ========================

/// Defines a shape based ona a starting point and an end point. Supports a glow effect.
///
/// At the moment this is implemented as a simple line that is rotated and translated to match
/// up with the start and end point.
/// FIXME make this nicer. For example, use splines.
mod shape {
    use super::*;

    ensogl::define_shape_system! {
         (start:Vector2<f32>, end:Vector2<f32>, thickness:f32, glow:f32) {

            let height     = Var::<f32>::from("input_start.y - input_end.y");
            let width      = Var::<f32>::from("input_start.x - input_end.x");
            let is_reverse = Var::<f32>::from("((((-sign(input_start.y-input_end.y)))))");

            let angle_inner = Var::<f32>::from(90_f32.to_radians());
            let triangle    = Triangle::<Var<f32>>::from_sides_and_angle(height,width,angle_inner);
            let angle_a     = Var::<Angle<Radians>>::from(is_reverse * triangle.angle_a().clone());

            let line         = Line(Var::<f32>::from(1.0) + thickness);
            let line         = line.rotate(&angle_a);
            let line_color   = Srgba::new(1.00, 0.565, 0.0, 1.0);
            let line_colored = line.fill(line_color);

            let glow           = Line(Var::<f32>::from(15.0) * glow);
            let glow           = glow.rotate(angle_a);
            let glow_color     = Srgb::new(1.00, 0.565, 0.0);
            let gradient_start = Srgba::new(glow_color.red,glow_color.green,glow_color.blue,0.0);
            let gradient_end   = Srgba::new(glow_color.red,glow_color.green,glow_color.blue,0.0);
            let glow_gradient  = LinearGradient::new()
                .add(0.0,gradient_start.into_linear())
                .add(1.0,gradient_end.into_linear());
            let glow_color     = SdfSampler::new(glow_gradient)
                .max_distance(15.0)
                .slope(Slope::Exponent(4.0));
            let glow_colored   = glow.fill(glow_color);

            (glow_colored + line_colored).into()
        }
    }
}



// ==============
// === Events ===
// ==============

/// Connection events.
///
/// `port_move`   should be emitted TO the `Connection`, so it can update its position and shape.
/// `hover_start` is emitted FROM the `Connection` when a mouse over event is detected.
/// `hover_end`   is emitted FROM the `Connection` when a mouse over event ends.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Events {
    pub network     : frp::Network,
    pub port_move   : frp::Source,
    pub hover_start : frp::Source,
    pub hover_end   : frp::Source,
}



// ==================
// === Connection ===
// ==================

/// Shape view for `Connection`.
///
/// Connections are center aligned to make some internal computations easier.
#[derive(Debug,Default,Clone,Copy)]
pub struct ConnectionView {}
impl ShapeViewDefinition for ConnectionView {
    type Shape = shape::Shape;
    fn new(_shape:&Self::Shape, _scene:&Scene,shape_registry:&ShapeRegistry) -> Self {
        let shape_system = shape_registry.shape_system(PhantomData::<shape::Shape>);
        shape_system.shape_system.set_alignment(
            alignment::HorizontalAlignment::Center,alignment::VerticalAlignment::Center);

        Self {}
    }
}

/// Connections represent a link between an `InputPort` and an `OutputPort`.
///
/// Ports are shown as a connecting line going from one port to the other. The position snd shape
/// of the `Connection` is defined by the start and end ports.
///
/// Connections can be created in the IDE by dragging from one port to another. This is currently handled in
/// the `GraphEditor`, where events are collected and routed to allow dragging from ports,
/// existence of a partially connected in-creation `Connection`, and finally the connecting of two
/// ports.
///
/// TODO: use two line segments. This reduces emtpy space and allows partial glow.
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
    start_port : RefCell<Option<WeakPort<OutputPortView>>>,
    /// Start of the connection.
    start      : Cell<Vector3<f32>>,
    end_port         : RefCell<Option<WeakPort<InputPortView>>>,
    /// End of the connection.
    end        : Cell<Vector3<f32>>,

    pub logger : Logger,
    pub events : Events,
    pub view   : Rc<component::ShapeView<ConnectionView>>,
    pub label  : frp::Source<String>,
}

impl Connection {
    /// Constructor.
    pub fn new() -> Self {
        frp::new_network! { connection_network
            def label       = source::<String> ();
            def port_move   = source::<()>     ();
            def hover_start = source::<()>     ();
            def hover_end   = source::<()>     ();
        }
        let network      = connection_network;
        let start        = Cell::new(Vector3::zero());
        let end          = Cell::new(Vector3::zero());
        let logger       = Logger::new("connection");
        let view         = Rc::new(component::ShapeView::new(&logger));
        let events       = Events{network,port_move,hover_start,hover_end};
        let start_port   = RefCell::new(None);
        let end_port     = RefCell::new(None);

        let data         = ConnectionData { start_port,start,end_port,end,label,events,view,
                                            logger };
        let data         = Rc::new(data);
        Self {data} . init() .init_frp()
    }

    fn init(self) -> Self {
        self
    }

    /// Set up the event handling for connections.
    ///
    /// Specifically:
    /// * translates mouse events in hover events.
    /// * sets up actions on port movement
    /// * sets up actions on hover events
    fn init_frp(self) -> Self {
        let weak_connection = self.downgrade();
        let network = &self.data.view.events.network;
        frp::new_bridge_network! { [network,self.data.events.network]
                let weak_connection_on_hover = weak_connection.clone();
                def _connection_on_over = self.data.view.events.mouse_over.map(f_!(() {
                    if let Some(connection) = weak_connection_on_hover.upgrade(){
                        connection.data.events.hover_start.emit(());
                    }
                }));

               let weak_connection_mouse_leave = weak_connection;
               def _connection_on_leave = self.data.view.events.mouse_leave.map(f_!(() {
                    if let Some(connection) = weak_connection_mouse_leave.upgrade(){
                        connection.data.events.hover_end.emit(());
                    }
                }));
        }
        frp::extend! { network
            let weak_connection = self.downgrade();
            def _f_on_port_move = self.data.events.port_move.map(move |_| {
               if let Some(connection) = weak_connection.upgrade(){
                    connection.on_port_position_change()
                }
            });
            let weak_connection_hover_start = self.downgrade();
            def _f_hover_start = self.data.events.hover_start.map(move |_| {
                 if let Some(connection) = weak_connection_hover_start.upgrade(){
                    connection.set_glow(1.0);
                }
            });
            let weak_connection_hover_end = self.downgrade();
             def _f_hover_end = self.data.events.hover_end.map(move |_| {
                 if let Some(connection) = weak_connection_hover_end.upgrade(){
                    connection.set_glow(0.0);
                }
            });
        }
        self
    }

    /// Sets the connection's glow.
    ///
    /// Should be a value between 0 and 1 to indicate glow strength.
    fn set_glow(&self, glow:f32) {
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.glow.set(glow);
        }
    }

    /// Helper function to update the `Connection`'s start position.
    fn set_start_position(&self, position:Vector3<f32>) {
        self.data.start.set(position);
        let start_pos = position - self.center();
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.start.set(start_pos.xy());
        }
        self.update_sprite();
    }

    /// Helper function to update the `Connection`'s end position.
    fn set_end_position(&self, position:Vector3<f32>) {
        self.data.end.set(position);
        let end_pos = position - self.data.start.get();
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.end.set(end_pos.xy());
        }
        self.update_sprite();
    }

    /// Set the connection's starting port.
    ///
    /// Will also  the shape and position of the `Connection` based on the port's position.
    pub fn set_start(&self, port:&OutputPort) {
        let position = port.position_global();
        self.data.start_port.set(port.downgrade());
        self.set_start_position(position);
    }

    /// Set the connection's end port.
    ///
    /// Will also update the shape and position of the `Connection` based on the port's position.
    pub fn set_end(&self, port:&InputPort) {
        let position = port.position_global();
        self.data.end_port.set(port.downgrade());
        self.set_end_position(position);
    }

    /// Sets the position of the connection start/end that has no port attached yet.
    ///
    /// This is mostly intended for dragging updates, which modify the unconnected end of the
    /// `Connection`.
    pub fn set_open_ends(&self, position: Vector3<f32>) {
        if self.data.start_port.borrow().is_none(){
            self.set_start_position(position);
        }
        if self.data.end_port.borrow().is_none(){
            self.set_end_position(position);
        }
    }

    /// Indicates whether this connection has both a start and an end port.
    pub fn fully_connected(&self) -> bool {
        self.data.start_port.borrow().is_some() && self.data.end_port.borrow().is_some()
    }

    /// Break the links to both start and end port.
    pub fn clear_ports(&self) {
        self.data.start_port
            .borrow_mut()
            .take()
            .map(|weak_port| weak_port.upgrade().map(| port| port.unset_connection()));
        self.data.end_port
            .borrow_mut()
            .take()
            .map(|weak_port| weak_port.upgrade().map(| port| port.unset_connection()));
    }


    /// Return the average between the start and end position.
    fn center(&self) -> Vector3<f32> {
        (self.data.end.get() + self.data.start.get()) * 0.5
    }

    /// Returns the dimensions of the rectangle that is constructed by start and end position.
    fn extent(&self) -> Vector3<f32> {
        (self.data.end.get() - self.data.start.get())
    }


    /// Helper function to update the sprite in response to start/end point changes.
    ///
    /// Ensures the sprite is centered between the start and end point, and large enough to
    /// encompass both. That allows us to draw a diagonal line from start to end.
    ///
    /// Note that this is somewhat wasteful at the moment: we have a mostly empty square sprite to
    /// render a line. But later on this might be filled by a curve, which requires this space.
    fn update_sprite(&self) {
        let center = self.center();
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.sprite.size().set(self.extent().xy());
        }
        self.data.view.display_object.set_position(Vector3::new(center.x,center.y,0.0));
    }

    /// Updates the connection end and start position with the respective port position.
    pub fn on_port_position_change(&self) {
        self.data.start_port
            .borrow()
            .as_ref()
            .map(|weak_port| weak_port
                .upgrade()
                .map(| port| self.set_start_position(port.position_global()))
            );
        self.data.end_port
            .borrow()
            .as_ref()
            .map(|weak_port| weak_port
                .upgrade()
                .map(| port| self.set_end_position(port.position_global()))
            );
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

/// Weak version of `Connection`.
#[derive(Clone,CloneRef,Debug)]
pub struct WeakConnection{
    data : Weak<ConnectionData>
}

impl StrongRef for Connection{
    type WeakRef = WeakConnection;
    fn downgrade(&self) -> WeakConnection {
        WeakConnection {data:Rc::downgrade(&self.data)}
    }
}

impl WeakRef for WeakConnection {
    type StrongRef = Connection;
    fn upgrade(&self) -> Option<Connection> {
        self.data.upgrade().map(|data| Connection{data})
    }
}
