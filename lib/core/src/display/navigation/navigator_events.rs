use super::mouse_manager::State;
use super::mouse_manager::MouseManager;
use super::mouse_manager::MouseClickEvent;
use super::mouse_manager::MouseWheelEvent;
use super::mouse_manager::MousePositionEvent;
use super::mouse_manager::MouseButton;
use crate::system::web::Result;
use crate::display::render::css3d::DOMContainer;

use nalgebra::Vector2;
use std::rc::Rc;
use std::cell::RefCell;
use nalgebra::zero;
use crate::display::navigation::mouse_manager::{MouseEventListener, WheelEventListener};

// =================
// === ZoomEvent ===
// =================

/// A struct holding zoom event information, such as the focus point and the amount of zoom.
pub struct ZoomEvent {
    pub focus  : Vector2<f32>,
    pub amount : f32
}

impl ZoomEvent {
    fn new(focus:Vector2<f32>, amount:f32, zoom_speed:f32) -> Self {
        let amount = amount * zoom_speed;
        Self { focus, amount }
    }

    fn from_mouse_wheel
    (event:MouseWheelEvent, mouse_position:Vector2<f32>, zoom_speed:f32) -> Self {
        let amount = event.movement_y;
        let focus  = mouse_position;
        Self::new(focus, amount, zoom_speed)
    }

    fn from_mouse_move(event:MousePositionEvent, focus:Vector2<f32>, zoom_speed:f32) -> Self {
        let amount = event.position.y - event.previous_position.y;
        Self::new(focus, amount, zoom_speed)
    }
}

pub trait FnZoomEvent = FnMut(ZoomEvent) + 'static;

// ================
// === PanEvent ===
// ================

/// A struct holding pan event information.
pub struct PanEvent {
    pub movement : Vector2<f32>
}

impl PanEvent {
    fn from(event:MousePositionEvent) -> Self {
        let mut movement   = event.position - event.previous_position;
                movement.x = -movement.x;
        Self { movement }
    }
}

pub trait FnPanEvent = FnMut(PanEvent) + 'static;



// ====================
// === MovementType ===
// ====================

#[derive(PartialEq)]
enum MovementType {
    Pan,
    Zoom { focus : Vector2<f32> }
}

// ===========================
// === NavigatorEventsData ===
// ===========================

struct NavigatorEventsData {
    movement_type    : Option<MovementType>,
    mouse_position   : Vector2<f32>,
    pan_callback     : Box<dyn FnPanEvent>,
    zoom_callback    : Box<dyn FnZoomEvent>,
    zoom_speed       : f32
}

impl NavigatorEventsData {
    fn new
    (pan_callback:Box<dyn FnPanEvent>, zoom_callback:Box<dyn FnZoomEvent>, zoom_speed:f32) -> Self {
        let mouse_position   = zero();
        let movement_type    = None;
        Self { mouse_position,movement_type,pan_callback,zoom_callback,zoom_speed }
    }

    fn call_zoom(&mut self, event:ZoomEvent) {
        (self.zoom_callback)(event);
    }

    fn call_pan(&mut self, event: PanEvent) {
        (self.pan_callback)(event);
    }
}

// =======================
// === NavigatorEvents ===
// =======================

/// Struct used to handle pan and zoom events from mouse interactions.
pub struct NavigatorEvents {
    mouse_manager : MouseManager,
    data          : Rc<RefCell<NavigatorEventsData>>,
    mouse_down    : Option<MouseEventListener>,
    mouse_move    : Option<MouseEventListener>,
    mouse_up      : Option<MouseEventListener>,
    mouse_leave   : Option<MouseEventListener>,
    wheel_zoom    : Option<WheelEventListener>
}

impl NavigatorEvents {
    pub fn new
    <P,Z>(dom:&DOMContainer, pan_callback:P, zoom_callback:Z, zoom_speed:f32) -> Result<Self>
    where P : FnPanEvent, Z : FnZoomEvent {
        let pan_callback      = Box::new(pan_callback);
        let zoom_callback     = Box::new(zoom_callback);
        let mouse_manager     = MouseManager::new(dom)?;
        let data              = NavigatorEventsData::new(pan_callback, zoom_callback, zoom_speed);
        let data              = Rc::new(RefCell::new(data));
        let mouse_move        = None;
        let mouse_up          = None;
        let mouse_leave       = None;
        let mouse_down        = None;
        let wheel_zoom        = None;
        let mut event_handler = Self {
            mouse_manager,
            data,
            mouse_down,
            mouse_move,
            mouse_up,
            mouse_leave,
            wheel_zoom
        };

        event_handler.initialize_events()?;
        Ok(event_handler)
    }

    // Initialize mouse events.
    fn initialize_events(&mut self) -> Result<()> {
        self.disable_context_menu()?;
        self.initialize_wheel_zoom()?;
        self.initialize_interaction_start_event()?;
        self.initialize_mouse_move_event()?;
        self.initialize_interaction_end_event()
    }

    fn initialize_wheel_zoom(&mut self) -> Result<()> {
        let data = self.data.clone();
        let listener = self.mouse_manager.add_mouse_wheel_callback(move |event:MouseWheelEvent| {
            let mut data       = data.borrow_mut();
            let mouse_position = data.mouse_position;
            let zoom_speed     = data.zoom_speed;
            let zoom_event     = ZoomEvent::from_mouse_wheel(event, mouse_position, zoom_speed);
            data.call_zoom(zoom_event);
        })?;
        self.wheel_zoom = Some(listener);
        Ok(())
    }

    fn initialize_interaction_start_event(&mut self) -> Result<()> {
        let data     = self.data.clone();
        let listener = self.mouse_manager.add_mouse_down_callback(move |event:MouseClickEvent| {
            match event.button {
                MouseButton::MIDDLE => {
                    data.borrow_mut().movement_type = Some(MovementType::Pan)
                },
                MouseButton::RIGHT => {
                    let focus = event.position;
                    data.borrow_mut().movement_type  = Some(MovementType::Zoom { focus })
                },
                _ => ()
            }
        })?;
        self.mouse_down = Some(listener);
        Ok(())
    }

    fn disable_context_menu(&mut self) -> Result<()> {
        self.mouse_manager.set_context_menu(State::Disabled)
    }

    fn initialize_interaction_end_event(&mut self) -> Result<()> {
        let data         = self.data.clone();
        let closure      = move |_| data.borrow_mut().movement_type = None;
        let listener     = self.mouse_manager.add_mouse_up_callback(closure)?;
        self.mouse_up    = Some(listener);

        let data         = self.data.clone();
        let closure      = move |_| data.borrow_mut().movement_type = None;
        let listener     = self.mouse_manager.add_mouse_leave_callback(closure)?;
        self.mouse_leave = Some(listener);
        Ok(())
    }

    fn initialize_mouse_move_event(&mut self) -> Result<()> {
        let data     = self.data.clone();
        let listener = self.mouse_manager.add_mouse_move_callback(move |event:MousePositionEvent| {
            let mut data = data.borrow_mut();
            data.mouse_position = event.position;

            if let Some(movement_type) = &data.movement_type {
                match movement_type {
                    MovementType::Zoom { focus } => {
                        let zoom_event = ZoomEvent::from_mouse_move(event, *focus, data.zoom_speed);
                        data.call_zoom(zoom_event);
                    },
                    MovementType::Pan => {
                        let pan_event = PanEvent::from(event);
                        data.call_pan(pan_event);
                    }
                }
            }
        })?;
        self.mouse_move = Some(listener);
        Ok(())
    }
}
