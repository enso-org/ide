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

/// A struct holding zoom event information.
pub struct ZoomEvent {
    pub focus  : Vector2<f32>,
    pub amount : f32
}

impl ZoomEvent {
    fn from_mouse_wheel(event:MouseWheelEvent, data:&Rc<RefCell<NavigatorEventsData>>) -> Self {
        let amount = event.movement_y;
        let focus  = data.borrow().mouse_position;
        Self { amount, focus }
    }

    fn from_mouse_move(event:MousePositionEvent, focus:Vector2<f32>) -> Self {
        let amount = event.position.y - event.previous_position.y;
        Self { focus, amount }
    }
}

pub trait FnZoomEvent = FnMut(ZoomEvent) + 'static;

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

pub trait FnPanningEvent = FnMut(PanningEvent) + 'static;



// ====================
// === MovementType ===
// ====================

#[derive(PartialEq)]
enum MovementType {
    Panning,
    Zooming { focus : Vector2<f32> }
}

// ===========================
// === NavigatorEventsData ===
// ===========================

struct NavigatorEventsData {
    movement_type    : Option<MovementType>,
    mouse_position   : Vector2<f32>,
    panning_callback : Box<dyn FnPanningEvent>,
    zoom_callback    : Box<dyn FnZoomEvent>
}

impl NavigatorEventsData {
    fn new(panning_callback:Box<dyn FnPanningEvent>, zoom_callback:Box<dyn FnZoomEvent>) -> Self {
        let mouse_position   = zero();
        let movement_type    = None;
        Self { mouse_position, movement_type, panning_callback, zoom_callback }
    }

    fn call_zoom(&mut self, event:ZoomEvent) {
        (self.zoom_callback)(event);
    }

    fn call_panning(&mut self, event:PanningEvent) {
        (self.panning_callback)(event);
    }
}

// =======================
// === NavigatorEvents ===
// =======================

/// Struct used to handle panning and zoom events from mouse interactions.
pub struct NavigatorEvents {
    mouse_manager : MouseManager,
    data          : Rc<RefCell<NavigatorEventsData>>,
    mouse_down    : Option<MouseEventListener>,
    mouse_move    : Option<MouseEventListener>,
    mouse_up      : Option<MouseEventListener>,
    mouse_leave   : Option<MouseEventListener>,
    wheel_zooming : Option<WheelEventListener>
}

impl NavigatorEvents {
    pub fn new<P,Z>(dom:&DOMContainer, panning_callback:P, zoom_callback:Z) -> Result<Self>
    where P : FnPanningEvent, Z : FnZoomEvent {
        let panning_callback  = Box::new(panning_callback);
        let zoom_callback     = Box::new(zoom_callback);
        let mouse_manager     = MouseManager::new(dom)?;
        let data              = NavigatorEventsData::new(panning_callback, zoom_callback);
        let data              = Rc::new(RefCell::new(data));
        let mouse_move        = None;
        let mouse_up          = None;
        let mouse_leave       = None;
        let mouse_down        = None;
        let wheel_zooming     = None;
        let mut event_handler = Self {
            mouse_manager,
            data,
            mouse_down,
            mouse_move,
            mouse_up,
            mouse_leave,
            wheel_zooming
        };

        event_handler.initialize_events()?;
        Ok(event_handler)
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
        let listener = self.mouse_manager.add_mouse_wheel_callback(move |event:MouseWheelEvent| {
            let zoom_event = ZoomEvent::from_mouse_wheel(event, &data);
            data.borrow_mut().call_zoom(zoom_event);
        })?;
        self.wheel_zooming = Some(listener);
        Ok(())
    }

    fn initialize_interaction_start_event(&mut self) -> Result<()> {
        let data     = self.data.clone();
        let listener = self.mouse_manager.add_mouse_down_callback(move |event:MouseClickEvent| {
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
                    MovementType::Zooming { focus } => {
                        let zoom_event = ZoomEvent::from_mouse_move(event, *focus);
                        data.call_zoom(zoom_event);
                    },
                    MovementType::Panning => {
                        let pan_event      = PanningEvent::from(event);
                        data.call_panning(pan_event);
                    }
                }
            }
        })?;
        self.mouse_move = Some(listener);
        Ok(())
    }
}