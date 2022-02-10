use crate::prelude::*;

use crate::control::callback;
use crate::control::io::mouse;
use crate::control::io::mouse::MouseManager;
use crate::display::navigation::navigator::SharedSwitch;

use nalgebra::zero;
use nalgebra::Vector2;



// =================
// === ZoomEvent ===
// =================

pub trait FnZoomEvent = FnMut(ZoomEvent) + 'static;

/// A struct holding zoom event information, such as the focus point and the amount of zoom.
pub struct ZoomEvent {
    pub focus:  Vector2<f32>,
    pub amount: f32,
}

impl ZoomEvent {
    fn new(focus: Vector2<f32>, amount: f32, zoom_speed: f32) -> Self {
        let amount = amount * zoom_speed;
        Self { focus, amount }
    }
}



// ================
// === PanEvent ===
// ================

pub trait FnPanEvent = FnMut(PanEvent) + 'static;

/// A struct holding pan event information.
pub struct PanEvent {
    pub movement: Vector2<f32>,
}

impl PanEvent {
    fn new(movement: Vector2<f32>) -> Self {
        Self { movement }
    }
}



// ====================
// === MovementType ===
// ====================

#[derive(PartialEq, Clone, Copy, Debug)]
enum MovementType {
    Pan,
    Zoom { focus: Vector2<f32> },
}



// =================================
// === NavigatorEventsProperties ===
// =================================

#[derive(Derivative)]
#[derivative(Debug)]
struct NavigatorEventsProperties {
    zoom_speed:          SharedSwitch<f32>,
    pan_speed:           SharedSwitch<f32>,
    disable_events:      Rc<Cell<bool>>,
    movement_type:       Option<MovementType>,
    last_mouse_position: Vector2<f32>,
    mouse_position:      Vector2<f32>,
    #[derivative(Debug = "ignore")]
    pan_callback:        Box<dyn FnPanEvent>,
    #[derivative(Debug = "ignore")]
    zoom_callback:       Box<dyn FnZoomEvent>,
}



// ===========================
// === NavigatorEventsData ===
// ===========================

#[derive(Debug)]
struct NavigatorEventsData {
    properties: RefCell<NavigatorEventsProperties>,
}

impl NavigatorEventsData {
    fn new(
        pan_callback: Box<dyn FnPanEvent>,
        zoom_callback: Box<dyn FnZoomEvent>,
        zoom_speed: SharedSwitch<f32>,
        pan_speed: SharedSwitch<f32>,
        disable_events: Rc<Cell<bool>>,
    ) -> Rc<Self> {
        let mouse_position = zero();
        let last_mouse_position = zero();
        let movement_type = None;
        let properties = RefCell::new(NavigatorEventsProperties {
            zoom_speed,
            pan_speed,
            disable_events,
            movement_type,
            last_mouse_position,
            mouse_position,
            pan_callback,
            zoom_callback,
        });
        Rc::new(Self { properties })
    }

    fn on_zoom(&self, event: ZoomEvent) {
        (&mut self.properties.borrow_mut().zoom_callback)(event);
    }

    fn on_pan(&self, event: PanEvent) {
        (&mut self.properties.borrow_mut().pan_callback)(event);
    }
}


// === Getters ===

impl NavigatorEventsData {
    fn mouse_position(&self) -> Vector2<f32> {
        self.properties.borrow().mouse_position
    }

    fn last_mouse_position(&self) -> Vector2<f32> {
        self.properties.borrow().last_mouse_position
    }

    fn zoom_speed(&self) -> f32 {
        self.properties.borrow().zoom_speed.get().into_on().unwrap_or(0.0)
    }

    fn pan_speed(&self) -> f32 {
        self.properties.borrow().pan_speed.get().into_on().unwrap_or(0.0)
    }

    fn movement_type(&self) -> Option<MovementType> {
        self.properties.borrow().movement_type
    }

    fn events_disabled(&self) -> bool {
        self.properties.borrow().disable_events.get()
    }
}


// === Setters ===

impl NavigatorEventsData {
    fn set_movement_type(&self, movement_type: Option<MovementType>) {
        self.properties.borrow_mut().movement_type = movement_type;
    }

    fn set_mouse_position(&self, mouse_position: Vector2<f32>) {
        let mut properties = self.properties.borrow_mut();
        properties.last_mouse_position = properties.mouse_position;
        properties.mouse_position = mouse_position;
    }
}


// =======================
// === NavigatorEvents ===
// =======================

/// Struct used to handle pan and zoom events from mouse interactions.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct NavigatorEvents {
    data:          Rc<NavigatorEventsData>,
    mouse_manager: MouseManager,
    #[derivative(Debug = "ignore")]
    mouse_down:    Option<callback::Handle>,
    #[derivative(Debug = "ignore")]
    mouse_up:      Option<callback::Handle>,
    #[derivative(Debug = "ignore")]
    mouse_move:    Option<callback::Handle>,
    #[derivative(Debug = "ignore")]
    mouse_leave:   Option<callback::Handle>,
    #[derivative(Debug = "ignore")]
    wheel_zoom:    Option<callback::Handle>,
}

impl NavigatorEvents {
    pub fn new<P, Z>(
        mouse_manager: &MouseManager,
        pan_callback: P,
        zoom_callback: Z,
        zoom_speed: SharedSwitch<f32>,
        pan_speed: SharedSwitch<f32>,
        disable_events: Rc<Cell<bool>>,
    ) -> Self
    where
        P: FnPanEvent,
        Z: FnZoomEvent,
    {
        let mouse_manager = mouse_manager.clone_ref();
        let pan_callback = Box::new(pan_callback);
        let zoom_callback = Box::new(zoom_callback);
        let mouse_move = default();
        let mouse_up = default();
        let mouse_down = default();
        let wheel_zoom = default();
        let mouse_leave = default();
        let data = NavigatorEventsData::new(
            pan_callback,
            zoom_callback,
            zoom_speed,
            pan_speed,
            disable_events,
        );
        let mut event_handler =
            Self { data, mouse_manager, mouse_down, mouse_up, mouse_move, mouse_leave, wheel_zoom };

        event_handler.initialize_mouse_events();
        event_handler
    }

    fn initialize_mouse_events(&mut self) {
        self.initialize_wheel_zoom();
        self.initialize_mouse_start_event();
        self.initialize_mouse_move_event();
        self.initialize_mouse_end_event();
    }

    fn initialize_wheel_zoom(&mut self) {
        let data = Rc::downgrade(&self.data);
        let listener = self.mouse_manager.on_wheel.add(move |event: &mouse::OnWheel| {
            if let Some(data) = data.upgrade() {
                if data.events_disabled() {
                    event.prevent_default();
                }
                if event.ctrl_key() {
                    // Prevent zoom event to be handed to the browser. This avoids browser scaling
                    // being applied to the whole IDE, thus we need to do this always when ctrl is
                    // pressed.
                    event.prevent_default();
                    let position = data.mouse_position();
                    let zoom_speed = data.zoom_speed();
                    let movement = Vector2::new(event.delta_x() as f32, -event.delta_y() as f32);
                    let amount = movement_to_zoom(movement);
                    let zoom_event = ZoomEvent::new(position, amount, zoom_speed);
                    data.on_zoom(zoom_event);
                } else {
                    let x = -event.delta_x() as f32;
                    let y = event.delta_y() as f32;
                    let pan_speed = data.pan_speed();
                    let movement = Vector2::new(x, y) * pan_speed;
                    let pan_event = PanEvent::new(movement);
                    data.on_pan(pan_event);
                }
            }
        });
        self.wheel_zoom = Some(listener);
    }

    fn initialize_mouse_start_event(&mut self) {
        let data = Rc::downgrade(&self.data);
        let listener = self.mouse_manager.on_down.add(move |event: &mouse::OnDown| {
            if let Some(data) = data.upgrade() {
                if data.events_disabled() {
                    event.prevent_default();
                }
                match event.button() {
                    mouse::MiddleButton => data.set_movement_type(Some(MovementType::Pan)),
                    mouse::SecondaryButton => {
                        let focus = event.position_relative_to_event_handler();
                        data.set_movement_type(Some(MovementType::Zoom { focus }))
                    }
                    _ => (),
                }
            }
        });
        self.mouse_down = Some(listener);
    }

    fn initialize_mouse_end_event(&mut self) {
        let data = Rc::downgrade(&self.data);
        let listener = self.mouse_manager.on_up.add(move |event: &mouse::OnUp| {
            if let Some(data) = data.upgrade() {
                if data.events_disabled() {
                    event.prevent_default();
                }
                data.set_movement_type(None);
            }
        });
        self.mouse_up = Some(listener);

        let data = Rc::downgrade(&self.data);
        let listener = self.mouse_manager.on_leave.add(move |event: &mouse::OnLeave| {
            if let Some(data) = data.upgrade() {
                if data.events_disabled() {
                    event.prevent_default();
                }
                data.set_movement_type(None);
            }
        });
        self.mouse_leave = Some(listener);
    }

    fn initialize_mouse_move_event(&mut self) {
        let data = Rc::downgrade(&self.data);
        let listener = self.mouse_manager.on_move.add(move |event: &mouse::OnMove| {
            if let Some(data) = data.upgrade() {
                if data.events_disabled() {
                    event.prevent_default();
                }

                let position = event.position_relative_to_event_handler();
                data.set_mouse_position(position);
                let movement = data.mouse_position() - data.last_mouse_position();

                if let Some(movement_type) = data.movement_type() {
                    match movement_type {
                        MovementType::Zoom { focus } => {
                            let zoom_speed = data.zoom_speed();
                            let zoom_amount = movement_to_zoom(movement);
                            let zoom_event = ZoomEvent::new(focus, zoom_amount, zoom_speed);
                            data.on_zoom(zoom_event);
                        }
                        MovementType::Pan => {
                            let pan_event = PanEvent::new(movement);
                            data.on_pan(pan_event);
                        }
                    }
                }
            }
        });
        self.mouse_move = Some(listener);
    }
}

fn movement_to_zoom(v: Vector2<f32>) -> f32 {
    let len = v.magnitude();
    // The zoom amount is a movement of camera along Z-axis, so positive values zoom out, and
    // negative zoom in.
    let sign = if v.x + v.y < 0.0 { 1.0 } else { -1.0 };
    sign * len
}
