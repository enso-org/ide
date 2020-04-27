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



// =========================
// === Connection Shapes ===
// =========================

mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (start:Vector2<f32>, end:Vector2<f32>, thickness:f32, glow:f32) {

            let height = Var::<f32>::from("input_start.y - input_end.y");
            let width  = Var::<f32>::from("input_start.x - input_end.x");

            let is_reverse : Var<f32> = "((((-sign(input_start.y-input_end.y)))))".into();

            let angle_inner = Var::<f32>::from(90_f32.to_radians());
            let triangle    = Triangle::<Var<f32>>::from_sides_and_angle(height,width,angle_inner);
            let angle_a     = Var::<Angle<Radians>>::from(is_reverse * triangle.angle_a().clone());

            let line         = Line(Var::<f32>::from(1.0) + thickness);
            let line         = line.rotate(&angle_a);
            let line_color   = Srgba::new(1.00, 0.565, 0.0, 1.0);
            let line_colored = line.fill(line_color);

            let glow          = Line(Var::<f32>::from(15.0) * glow);
            let glow          = glow.rotate(angle_a);
            let glow_color    = Srgb::new(1.00, 0.565, 0.0);
            let glow_gradient = LinearGradient::new()
                .add(0.0,Srgba::new(glow_color.red,glow_color.green,glow_color.blue,0.0).into_linear())
                .add(1.0,Srgba::new(glow_color.red,glow_color.green,glow_color.blue,1.0).into_linear());
            let glow_color    = SdfSampler::new(glow_gradient).max_distance(15.0).slope(Slope::Exponent(4.0));            let glow_colored = glow.fill(line_color);
            let glow_colored          = glow.fill(glow_color);

            (glow_colored + line_colored).into()
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
    pub network     : frp::Network,
    pub select      : frp::Source,
    pub deselect    : frp::Source,
    pub port_move   : frp::Source,
    pub hover_start : frp::Source,
    pub hover_end   : frp::Source,
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
        shape.start.set(Vector2::zero());
        shape.end.set(Vector2::zero());

        let bbox = Vector2::new(200.0, 200.0);
        shape.sprite.size().set(bbox);

        let shape_system = shape_registry.shape_system(PhantomData::<shape::Shape>);
        shape_system.shape_system.set_alignment(alignment::HorizontalAlignment::Center,alignment::VerticalAlignment::Center);

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
    start_port       : RefCell<Option<WeakPort<OutputPortView>>>,
    /// Start of the connection.
    start       : Cell<Vector3<f32>>,
    end_port         : RefCell<Option<WeakPort<InputPortView>>>,
    /// End of the connection.
    end         : Cell<Vector3<f32>>,

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
            def select      = source::<()>     ();
            def deselect    = source::<()>     ();
            def port_move   = source::<()>     ();
            def hover_start = source::<()>     ();
            def hover_end   = source::<()>     ();

        }
        let network      = connection_network;
        let start        = Cell::new(Vector3::zero());
        let end          = Cell::new(Vector3::zero());
        let logger       = Logger::new("connection");
        let view         = Rc::new(component::ShapeView::new(&logger));
        let events       = Events{network,select,deselect,port_move,hover_start,hover_end};
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
        let weak_connection = self.downgrade();
        frp::extend! { network
            let weak_connection = weak_connection.clone();
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
    pub fn set_glow(&self, glow:f32) {
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.glow.set(glow);
        }
    }

    /// Set the connections origin.
    pub fn set_start(&self, port:&OutputPort) {
        let position = port.position_global();
        self.data.start_port.set(port.downgrade());
        self.set_start_position(position);
    }

    fn set_start_position(&self, position:Vector3<f32>) {
        self.data.start.set(position);
        let start_pos = position - self.center();
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.start.set(start_pos.xy());
        }
        self.update_sprite();
    }

    /// Set the position where the connections ends.
    pub fn set_end(&self, port:&InputPort) {
        let position = port.position_global();
        self.data.end_port.set(port.downgrade());
        self.set_end_position(position);
    }

    fn set_end_position(&self, position:Vector3<f32>) {
        self.data.end.set(position);
        let end_pos = position - self.data.start.get();
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.end.set(end_pos.xy());
        }
        self.update_sprite();
    }

    /// Sets the position of the connection start/end that has no port attached yet.
    pub fn set_open_ends(&self, position: Vector3<f32>){
        if self.data.start_port.borrow().is_none(){
            self.set_start_position(position);
        }
        if self.data.end_port.borrow().is_none(){
            self.set_end_position(position);
        }
    }

    /// Indicates whether this connection has both a start and an end port.
    pub fn fully_connected(&self) -> bool{
        self.data.start_port.borrow().is_some() && self.data.end_port.borrow().is_some()
    }

    /// Break the link to both start and end port.
    pub fn clear_ports(&self){
        self.data.start_port
            .borrow_mut()
            .take()
            .map(|weak_port| weak_port.upgrade().map(| port| port.unset_connection()));
        self.data.end_port
            .borrow_mut()
            .take()
            .map(|weak_port| weak_port.upgrade().map(| port| port.unset_connection()));
    }

    fn center(&self) -> Vector3<f32> {
        (self.data.end.get() + self.data.start.get()) * 0.5
    }

    fn extent(&self) -> Vector3<f32> {
        (self.data.end.get() - self.data.start.get())
    }

    fn update_sprite(&self) {
        let center = self.center();
        if let Some(t) = self.data.view.data.borrow().as_ref() {
            t.shape.sprite.size().set(self.extent().xy());
        }
        self.data.view.display_object.set_position(Vector3::new(center.x,center.y,0.0));
    }

    /// Updates the connection end and start position with the respective port position.
    pub fn on_port_position_change(&self){
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
    fn downgrade(&self) -> WeakConnection{
        WeakConnection {data:Rc::downgrade(&self.data)}
    }
}

impl WeakRef for WeakConnection {
    type StrongRef = Connection;
    fn upgrade(&self) -> Option<Connection> {
        self.data.upgrade().map(|data| Connection{data})
    }
}

