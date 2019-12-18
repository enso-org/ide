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

// ====================
// === ZoomingEvent ===
// ====================

/// A struct holding zooming event information.
pub struct ZoomingEvent {
    pub focus  : Vector2<f32>,
    pub amount : f32
}

impl ZoomingEvent {
    fn from_mouse_wheel(event:MouseWheelEvent, data:&Rc<RefCell<CameraManagerData>>) -> Self {
        let amount = event.movement_y;
        let focus  = data.borrow().mouse_position;
        Self { amount, focus }
    }

    fn from_mouse_move(event:MousePositionEvent, focus:Vector2<f32>) -> Self {
        let amount = event.position.y - event.previous_position.y;
        Self { focus, amount }
    }
}

// ====================
// === PanningEvent ===
// ====================

/// A struct holding panning event information.
pub struct PanningEvent {
    pub movement : Vector2<f32>
}

impl PanningEvent {
    fn from(event:MousePositionEvent) -> Self {
        let mut movement   = event.position - event.previous_position;
                movement.x = -movement.x;
        Self { movement }
    }
}

// =============
// === Event ===
// =============

/// An enumeration representing navigation events.
pub enum Event {
    Zooming(ZoomingEvent),
    Panning(PanningEvent)
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

struct CameraManagerData {
    event_queue       : Vec<Event>,
    movement_type     : Option<MovementType>,
    mouse_position    : Vector2<f32>
}

impl Default for CameraManagerData {
    fn default() -> Self {
        let event_queue       = default();
        let mouse_position    = zero();
        let movement_type     = None;
        Self { event_queue, mouse_position, movement_type }
    }
}

// =====================
// === CameraManager ===
// =====================

/// Struct used to handle mouse events such as MouseDown, MouseMove, MouseUp,
/// MouseLeave and MouseWheel.
pub struct CameraManager {
    mouse_manager : MouseManager,
    data          : Rc<RefCell<CameraManagerData>>
}

impl CameraManager {
    pub fn new(dom:&DOMContainer) -> Result<Self> {
        let mouse_manager     = MouseManager::new(dom)?;
        let data              = default();
        let data              = Rc::new(RefCell::new(data));
        let mut event_handler = Self { mouse_manager, data };

        event_handler.initialize_events()?;
        Ok(event_handler)
    }

    /// Checks if EventHandler has an event.
    pub fn poll(&mut self) -> Option<Event> {
        self.data.borrow_mut().event_queue.pop()
    }

    // Initialize mouse events.
    fn initialize_events(&mut self) -> Result<()> {
        self.disable_context_menu()?;
        self.initialize_wheel_zooming()?;
        self.initialize_interaction_start_event()?;
        self.initialize_mouse_move_event()?;
        self.initialize_interaction_end_event()
    }

    fn initialize_wheel_zooming(&mut self) -> Result<()> {
        let data = self.data.clone();
        self.mouse_manager.add_mouse_wheel_callback(move |event:MouseWheelEvent| {
            let zoom_event = ZoomingEvent::from_mouse_wheel(event, &data);
            let event      = Event::Zooming(zoom_event);
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
                        let zoom_event = ZoomingEvent::from_mouse_move(event, *focus);
                        let event      = Event::Zooming(zoom_event);
                        data.event_queue.push(event);
                    },
                    MovementType::Panning => {
                        let pan_event      = PanningEvent::from(event);
                        let event          = Event::Panning(pan_event);
                        data.event_queue.push(event);
                    }
                }
            }
        })?.forget();
        Ok(())
    }
}