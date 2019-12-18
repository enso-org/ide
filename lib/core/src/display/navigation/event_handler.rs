// FIXME: NEEDS MAJOR REFACTORING!

use crate::prelude::*;

use super::mouse_manager::State;
use super::mouse_manager::MouseManager;
use super::mouse_manager::MouseClickEvent;
use super::mouse_manager::MouseWheelEvent;
use super::mouse_manager::MousePositionEvent;
use super::mouse_manager::MouseButton;
use crate::system::web::Result;
use crate::display::rendering::DOMContainer;

use nalgebra::Vector2;
use std::rc::Rc;
use std::cell::RefCell;
use nalgebra::zero;

// =================
// === ZoomEvent ===
// =================

pub struct ZoomEvent {
    pub focus  : Vector2<f32>,
    pub amount : f32
}

// ================
// === PanEvent ===
// ================

pub struct PanEvent {
    pub movement : Vector2<f32>
}

// =============
// === Event ===
// =============

/// Navigation user interactions events.
pub enum Event {
    Zoom(ZoomEvent),
    Pan(PanEvent)
}

// ====================
// === MovementType ===
// ====================

#[derive(PartialEq)]
enum MovementType {
    Panning,
    Zooming { focus : Vector2<f32> }
}

// ========================
// === EventHandlerData ===
// ========================

struct EventHandlerData {
    event_queue    : Vec<Event>,
    movement_type  : Option<MovementType>,
    mouse_position : Vector2<f32>
}

impl Default for EventHandlerData {
    fn default() -> Self {
        let event_queue    = default();
        let mouse_position = zero();
        let movement_type  = None;
        Self { event_queue, mouse_position, movement_type }
    }
}

// ====================
// === EventHandler ===
// ====================

/// Struct used to handle mouse events such as MouseDown, MouseMove, MouseUp,
/// MouseLeave and MouseWheel.
pub struct EventHandler {
    mouse_manager : MouseManager,
    data          : Rc<RefCell<EventHandlerData>>
}

impl EventHandler {
    pub fn new(dom:&DOMContainer) -> Result<Self> {
        let mouse_manager     = MouseManager::new(dom)?;
        let data              = default();
        let data              = Rc::new(RefCell::new(data));
        let mut event_handler = Self { mouse_manager, data };

        event_handler.initialize_events()?;
        Ok(event_handler)
    }

    fn initialize_wheel_zooming(&mut self) -> Result<()> {
        let data = self.data.clone();
        self.mouse_manager.add_mouse_wheel_callback(move |event:MouseWheelEvent| {
            let amount     = event.movement_y;
            let focus      = data.borrow().mouse_position;
            let zoom_event = ZoomEvent { amount, focus };
            let event      = Event::Zoom(zoom_event);
            data.borrow_mut().event_queue.push(event);
        })?.forget();
        Ok(())
    }

    fn initialize_interaction_start_event(&mut self) -> Result<()> {
        let data = self.data.clone();
        self.mouse_manager.add_mouse_down_callback(move |event:MouseClickEvent| {
            match event.button {
                MouseButton::MIDDLE => {
                    data.borrow_mut().movement_type = Some(MovementType::Panning)
                },
                MouseButton::RIGHT => {
                    let focus = event.position;
                    data.borrow_mut().movement_type  = Some(MovementType::Zooming { focus })
                },
                _ => ()
            }
        })?.forget();
        Ok(())
    }

    fn disable_context_menu(&mut self) -> Result<()> {
        self.mouse_manager.set_context_menu(State::Disabled)?;
        self.mouse_manager.set_context_menu(State::Enabled)?;
        self.mouse_manager.set_context_menu(State::Disabled)
    }

    fn initialize_interaction_end_event(&mut self) -> Result<()> {
        let data    = self.data.clone();
        let closure = move |_| data.borrow_mut().movement_type = None;
        self.mouse_manager.add_mouse_up_callback(closure)?.forget();

        let data = self.data.clone();
        let closure = move |_| data.borrow_mut().movement_type = None;
        self.mouse_manager.add_mouse_leave_callback(closure)?.forget();
        Ok(())
    }

    fn initialize_mouse_move_event(&mut self) -> Result<()> {
        let data = self.data.clone();
        self.mouse_manager.add_mouse_move_callback(move |event:MousePositionEvent| {
            let mut data = data.borrow_mut();
            data.mouse_position = event.position;

            if let Some(movement_type) = &data.movement_type {
                match movement_type {
                    MovementType::Zooming { focus } => {
                        let focus      = *focus;
                        let amount     = event.position.y - event.previous_position.y;
                        let zoom_event = ZoomEvent { focus, amount };
                        let event      = Event::Zoom(zoom_event);
                        data.event_queue.push(event);
                    },
                    MovementType::Panning => {
                        let mut movement   = event.position - event.previous_position;
                        movement.x = -movement.x;
                        let pan_event      = PanEvent { movement };
                        let event          = Event::Pan(pan_event);
                        data.event_queue.push(event);
                    }
                }
            }
        })?.forget();
        Ok(())
    }

    /// Initialize mouse events.
    fn initialize_events(&mut self) -> Result<()> {
        self.disable_context_menu()?;
        self.initialize_wheel_zooming()?;
        self.initialize_interaction_start_event()?;
        self.initialize_mouse_move_event()?;
        self.initialize_interaction_end_event()
    }

    /// Checks if EventHandler has an event.
    pub fn poll(&mut self) -> Option<Event> {
        self.data.borrow_mut().event_queue.pop()
    }
}