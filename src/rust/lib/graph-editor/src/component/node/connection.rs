//! This module defines the shapes and logic required for drawing node connections.
use crate::prelude::*;

use crate::frp;
use crate::component::node::port::InputPort;
use crate::component::node::port::InputPortView;
use crate::component::node::port::OutputPort;
use crate::component::node::port::OutputPortView;
use crate::component::node::port::WeakPort;

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
use ensogl::gui::component::animation;
use ensogl::gui::component;


// ========================
// === Connection Shape ===
// ========================

/// Defines connection shape with glow effect. At the moment this is implemented as a simple line.
mod shape {
    use super::*;

    ensogl::define_shape_system! {
         (thickness:f32, glow:f32) {
            let zoom_factor  = Var::<f32>::from("(2.0 / input_zoom) - 1.0");

            let line_thickness = (Var::<f32>::from(1.0) + thickness) * zoom_factor;
            let line           = Line(&line_thickness);
            let line_color     = Srgba::new(1.00, 0.565, 0.0, 1.0);
            let line_colored   = line.fill(line_color);

            let glow           = Line(Var::<f32>::from(75.0) * &line_thickness * glow);
            let glow_color     = Srgb::new(1.00, 0.565, 0.0);
            let gradient_start = Srgba::new(glow_color.red,glow_color.green,glow_color.blue,0.0);
            let gradient_end   = Srgba::new(glow_color.red,glow_color.green,glow_color.blue,0.65);
            let glow_gradient  = LinearGradient::new()
                .add(0.0,gradient_start.into_linear())
                .add(1.0,gradient_end.into_linear());
            let glow_color     = SdfSampler::new(glow_gradient)
                .max_distance(100.0)
                .slope(Slope::Exponent(4.0));
            let glow_colored   = glow.fill(glow_color);

            /// Add an almost invisible area extend input area.
            let touch_extension        = Line(Var::<f32>::from(2.0) * &line_thickness);
            let touch_extension        = touch_extension.fill(Srgba::new(1.0,1.0,1.0,0.001).into_linear());

            (touch_extension + glow_colored + line_colored).into()
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
    pub network         : frp::Network,
    pub hover_start     : frp::Source,
    pub hover_end       : frp::Source,
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

type ConnectionShapeView = component::ShapeView<ConnectionView>;

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
    input_port: RefCell<Option<WeakPort<InputPortView>>>,
    /// Start of the connection.
    start      : Cell<Vector3<f32>>,
    output_port: RefCell<Option<WeakPort<OutputPortView>>>,
    /// End of the connection.
    end        : Cell<Vector3<f32>>,

    pub logger : Logger,
    pub events : Events,
    pub view   : Rc<ConnectionShapeView>,
    pub label  : frp::Source<String>,
}

impl Connection {
    /// Constructor.
    pub fn new() -> Self {
        frp::new_network! { connection_network
            def label           = source::<String> ();
            def hover_start     = source::<()>     ();
            def hover_end       = source::<()>     ();
        }
        let network      = connection_network;
        let start        = Cell::new(Vector3::zero());
        let end          = Cell::new(Vector3::zero());
        let logger       = Logger::new("connection");
        let view         = Rc::new(component::ShapeView::new(&logger));
        let events       = Events{network,hover_start,hover_end};
        let start_port   = RefCell::new(None);
        let end_port     = RefCell::new(None);

        let data         = ConnectionData {
            input_port: start_port,start,
            output_port: end_port,end,label,events,view,logger };
        let data         = Rc::new(data);
        Self {data} . init_frp()
    }

    /// Create a new connection between the two ports.
    pub fn with_ports(input_port: &InputPort, output_port: &OutputPort) -> Self {
        let connection = Connection::new();
        connection.set_input_port(input_port);
        connection.set_output_port(output_port);
        connection
    }

    /// Set up the event handling for connections.
    fn init_frp(self) -> Self {
        let weak_connection = self.downgrade();

        let network = &self.data.events.network;
        frp::extend! { network
                def _connection_on_over = self.data.view.events.mouse_over.map(f!((weak_connection)(_) {
                    if let Some(connection) = weak_connection.upgrade(){
                        connection.data.events.hover_start.emit(());
                    }
                }));

               def _connection_on_leave = self.data.view.events.mouse_leave.map(f!((weak_connection)(_) {
                    if let Some(connection) = weak_connection.upgrade(){
                        connection.data.events.hover_end.emit(());
                    }
                }));
        }

        let network = &self.data.view.events.network;
        let weak_connection_fade = self.downgrade();
        let fade = animation(network,move |value| {
            weak_connection_fade.upgrade().for_each(|connection| {
                connection.set_glow(value);
            })
        });

        frp::extend! { network
            def _f_hover_start = self.data.events.hover_start.map(enclose!((fade) move |_| {
                    fade.set_position(1.0);
                    // TODO should be removed once mouse_leave is implemented.
                    fade.set_target_position(0.0);
            }));
             def _f_hover_end = self.data.events.hover_end.map(enclose!((fade) move |_| {
                    fade.set_target_position(0.0);
            }));
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

    /// Returns the position of the input end of the connection.
    pub fn input_position(&self) -> Vector3<f32> {
        self.data.start.get()
    }

    /// Helper function to update the input position and update the shape.
    fn set_input_position(&self, position:Vector3<f32>) {
        self.data.start.set(position);
        self.update_sprite();
    }

    /// Returns the position of the output end of the connection.
    pub fn output_position(&self) -> Vector3<f32> {
        self.data.end.get()
    }

    /// Helper function to update the output position and update the shape.
    fn set_output_position(&self, position:Vector3<f32>) {
        self.data.end.set(position);
        self.update_sprite();
    }

    /// Set the connection's starting port.
    ///
    /// Will also  the shape and position of the `Connection` based on the port's position.
    pub fn set_input_port(&self, port:&InputPort) {
        let position = port.position_global();
        self.data.input_port.set(port.downgrade());
        self.set_input_position(position);
    }

    /// Set the connection's end port.
    ///
    /// Will also update the shape and position of the `Connection` based on the port's position.
    pub fn set_output_port(&self, port:&OutputPort) {
        let position = port.position_global();
        self.data.output_port.set(port.downgrade());
        self.set_output_position(position);
    }

    /// Sets the position of the connection start/end that has no port attached yet.
    ///
    /// This is mostly intended for dragging updates, which modify the unconnected end of the
    /// `Connection`.
    pub fn set_open_ends(&self, position: Vector3<f32>) {
        if self.data.input_port.borrow().is_none(){
            self.set_input_position(position);
            self.on_input_port_position_change()
        }
        if self.data.output_port.borrow().is_none(){
            self.set_output_position(position);
            self.on_output_port_position_change()
        }
    }

    /// Indicates whether this connection has both a start and an end port attached.
    pub fn fully_connected(&self) -> bool {
        self.data.input_port.borrow().is_some() && self.data.output_port.borrow().is_some()
    }

    /// Remove the port and inform the port that the connection was severed.
    pub fn clear_input_port(&self) {
        self.data.input_port
            .borrow_mut()
            .take()
            .map(|weak_port| weak_port.upgrade().map(| port| port.clear_connection()));
    }

    /// Remove the port and inform the port that the connection was severed.
    pub fn clear_output_port(&self) {
        self.data.output_port
            .borrow_mut()
            .take()
            .map(|weak_port| weak_port.upgrade().map(| port| port.clear_connection()));
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
    /// Ensures the sprite is centered between the start and end point and correctly rotated so
    /// start and end point match up.
    fn update_sprite(&self) {
        let center = self.center();
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            let extent = self.extent().xy();
            let diagonal = (extent.x * extent.x + extent.y * extent.y).sqrt();
            t.shape.sprite.size().set(Vector2::new(diagonal, 200.0));
        }
        self.data.view.display_object.set_position(Vector3::new(center.x,center.y,0.0));

        let up     = Vector3::new(1.0,0.0,0.0);
        let target =  self.output_position() - self.input_position();
        let delta  = up - target;
        let angle  = ( delta.y ).atan2( delta.x );

        self.data.view.display_object.set_rotation(Vector3::new(0.0,0.0,angle));
    }

    /// Needs to be called when the input port's position has changed to propagate the layout
    /// update to the output port.
    pub fn on_input_port_position_change(&self) {
        self.data.input_port
            .borrow()
            .as_ref()
            .map(|weak_port| weak_port
                .upgrade()
                .map(| port| {
                    self.set_input_position(port.connection_position())
                })
            );
        self.data.output_port
            .borrow()
            .as_ref()
            .map(|weak_port| weak_port
                .upgrade()
                .map(| port| {
                    self.set_output_position(port.connection_position());
                    port.on_connection_update()
                })
            );
    }

    /// Needs to be called when the output port's position has changed to propagate the layout
    /// update to the input port.
    pub fn on_output_port_position_change(&self) {
        self.data.output_port
            .borrow()
            .as_ref()
            .map(|weak_port| weak_port
                .upgrade()
                .map(| port| {
                    self.set_output_position(port.connection_position())
                })
            );
        self.data.input_port
            .borrow()
            .as_ref()
            .map(|weak_port| weak_port
                .upgrade()
                .map(| port| {
                    self.set_input_position(port.connection_position());
                    port.on_connection_update()
                })
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
