use crate::display::render::css3d::DOMContainer;
use crate::system::web::dyn_into;
use crate::system::web::Result;
use crate::system::web::Error;
use crate::system::web::ignore_context_menu;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;
use web_sys::WheelEvent;
use web_sys::EventTarget;
use web_sys::AddEventListenerOptions;
use js_sys::Function;
use nalgebra::Vector2;
use std::rc::Rc;
use std::cell::RefCell;



// ======================
// === EventListeners ===
// ======================

/// A mouse event listener object returned when adding callbacks to MouseManager.
pub type MouseEventListener = Closure<dyn Fn(MouseEvent)>;
/// A wheel event listener object returned when adding callbacks to MouseManager.
pub type WheelEventListener = Closure<dyn Fn(WheelEvent)>;



// ===================
// === MouseButton ===
// ===================

/// An enumeration representing the mouse buttons.
pub enum MouseButton {
    LEFT,
    MIDDLE,
    RIGHT
}



// =======================
// === MouseClickEvent ===
// =======================

/// A struct storing information about mouse down and mouse up events.
pub struct MouseClickEvent {
    pub position : Vector2<f32>,
    pub button   : MouseButton
}

impl MouseClickEvent {
    fn from(event:MouseEvent, data:&Rc<RefCell<MouseManagerData>>) -> Self {
        let position  = Vector2::new(event.x() as f32, event.y() as f32);
        let position  = position - data.borrow().dom.position();
        let button    = match event.button() {
            LEFT_MOUSE_BUTTON      => MouseButton::LEFT,
            MIDDLE_MOUSE_BUTTON    => MouseButton::MIDDLE,
            RIGHT_MOUSE_BUTTON | _ => MouseButton::RIGHT
        };
        Self { position, button }
    }
}

pub trait FnMouseClick = Fn(MouseClickEvent) + 'static;



// ==========================
// === MousePositionevent ===
// ==========================

/// A struct storing information about mouse move, mouse enter and mouse leave events.
pub struct MousePositionEvent {
    pub previous_position : Vector2<f32>,
    pub position          : Vector2<f32>
}

impl MousePositionEvent {
    fn from(event:MouseEvent, data:&Rc<RefCell<MouseManagerData>>) -> Self {
        let position  = Vector2::new(event.x() as f32, event.y() as f32);
        let position  = position - data.borrow().dom.position();
        let previous_position = match data.borrow().mouse_position {
            Some(position) => position,
            None           => position
        };
        data.borrow_mut().mouse_position = Some(position);
        Self { previous_position, position }
    }
}

pub trait FnMousePosition = Fn(MousePositionEvent)  + 'static;



// =======================
// === MouseWheelEvent ===
// =======================

/// A struct storing information about mouse wheel events.
pub struct MouseWheelEvent {
    pub movement_y : f32
}

impl MouseWheelEvent {
    fn from(event:WheelEvent) -> Self {
        let movement_y = event.delta_y() as f32;
        Self { movement_y }
    }
}

pub trait FnMouseWheel = Fn(MouseWheelEvent) + 'static;



// ========================
// === MouseManagerData ===
// ========================

/// A struct used for storing shared MouseManager's mutable data.
struct MouseManagerData {
    dom            : DOMContainer,
    mouse_position : Option<Vector2<f32>>
}



// =============
// === State ===
// =============

/// An enum mainly used for enabling or disabling the Context Menu.
pub enum State {
    Enabled,
    Disabled
}



// ====================
// === MouseManager ===
// ====================

/// MouseManager
pub struct MouseManager {
    target : EventTarget,
    data   : Rc<RefCell<MouseManagerData>>,
    ignore_context_menu : Option<MouseEventListener>,
    stop_mouse_tracking : Option<MouseEventListener>
}

const   LEFT_MOUSE_BUTTON: i16 = 0;
const MIDDLE_MOUSE_BUTTON: i16 = 1;
const  RIGHT_MOUSE_BUTTON: i16 = 2;

impl MouseManager {
    pub fn new(dom:&DOMContainer) -> Result<Self> {
        let target              = dyn_into::<_, EventTarget>(dom.dom.clone())?;
        let dom                 = dom.clone();
        let mouse_position      = None;
        let data                = MouseManagerData { dom, mouse_position };
        let data                = Rc::new(RefCell::new(data));
        let ignore_context_menu = None;
        let stop_mouse_tracking = None;
        let mut mouse_manager   = Self { target, data, ignore_context_menu, stop_mouse_tracking };
        mouse_manager.stop_tracking_mouse_when_it_leaves_dom()?;
        Ok(mouse_manager)
    }

    /// Sets context menu state to enabled or disabled.
    pub fn set_context_menu(&mut self, state:State) -> Result<()> {
        match state {
            State::Enabled => {
                if let Some(callback) = &self.ignore_context_menu {
                    let callback = callback.as_ref().unchecked_ref();
                    remove_event_listener_with_callback(&self.target, "contextmenu", callback)?;
                    self.ignore_context_menu = None;
                }
            },
            State::Disabled => {
                if self.ignore_context_menu.is_none() {
                    self.ignore_context_menu = Some(ignore_context_menu(&self.target)?);
                };
            }
        }
        Ok(())
    }

    /// Adds mouse down event callback and returns its listener object.
    pub fn add_mouse_down_callback<F:FnMouseClick>(&mut self, f:F) -> Result<MouseEventListener> {
        let data = self.data.clone();
        let closure = move |event:MouseEvent| f(MouseClickEvent::from(event, &data));
        add_mouse_event(&self.target, "mousedown", closure)
    }

    /// Removes the mouse down event listener.
    pub fn remove_mouse_down_callback(&mut self, listener:MouseEventListener) -> Result<()> {
        remove_mouse_event(&self.target, "mousedown", listener)
    }

    /// Adds mouse up event callback and returns its listener object.
    pub fn add_mouse_up_callback<F:FnMouseClick>(&mut self, f:F) -> Result<MouseEventListener> {
        let data = self.data.clone();
        let closure = move |event:MouseEvent| f(MouseClickEvent::from(event, &data));
        add_mouse_event(&self.target, "mouseup", closure)
    }

    /// Removes mouse up event event listener.
    pub fn remove_mouse_up_callback(&mut self, listener:MouseEventListener) -> Result<()> {
        remove_mouse_event(&self.target, "mouseup", listener)
    }

    /// Adds mouse move event callback and returns its listener object.
    pub fn add_mouse_move_callback
    <F:FnMousePosition>(&mut self, f:F) -> Result<MouseEventListener> {
        let data = self.data.clone();
        let closure = move |event:MouseEvent| f(MousePositionEvent::from(event, &data));
        add_mouse_event(&self.target, "mousemove", closure)
    }

    /// Removes mouse move event listener.
    pub fn remove_mouse_move_callback(&mut self, listener:MouseEventListener) -> Result<()> {
        remove_mouse_event(&self.target, "mousemove", listener)
    }

    /// Adds mouse leave event callback and returns its listener object.
    pub fn add_mouse_leave_callback
    <F:FnMousePosition>(&mut self, f:F) -> Result<MouseEventListener> {
        let data = self.data.clone();
        let closure = move |event:MouseEvent| f(MousePositionEvent::from(event, &data));
        add_mouse_event(&self.target, "mouseleave", closure)
    }

    /// Removes mouse leave event listener.
    pub fn remove_mouse_leave_callback(&mut self, listener:MouseEventListener) -> Result<()> {
        remove_mouse_event(&self.target, "mouseleave", listener)
    }

    /// Adds MouseWheel event callback and returns its listener object.
    pub fn add_mouse_wheel_callback<F:FnMouseWheel>(&mut self, f:F) -> Result<WheelEventListener> {
        let closure = move |event:WheelEvent| f(MouseWheelEvent::from(event));
        add_wheel_event(&self.target, closure)
    }

    /// Removes MouseWheel event listener.
    pub fn remove_mouse_wheel_callback(&mut self, listener:WheelEventListener) -> Result<()> {
        remove_wheel_event(&self.target, listener)
    }

    fn stop_tracking_mouse_when_it_leaves_dom(&mut self) -> Result<()> {
        let data    = self.data.clone();
        let closure = move |_| data.borrow_mut().mouse_position = None;
        self.stop_mouse_tracking = Some(add_mouse_event(&self.target, "mouseleave", closure)?);
        Ok(())
    }
}



// =============
// === Utils ===
// =============

fn add_event_listener_with_callback
(target:&EventTarget, name:&str, function:&Function) -> Result<()> {
    match target.add_event_listener_with_callback(name, function) {
        Ok(_)  => Ok(()),
        Err(_) => Err(Error::FailedToAddEventListener)
    }
}

fn remove_event_listener_with_callback
(target:&EventTarget, name:&str, function:&Function) -> Result<()> {
    match target.remove_event_listener_with_callback(name, function) {
        Ok(_)  => Ok(()),
        Err(_) => Err(Error::FailedToRemoveEventListener)
    }
}

/// Adds mouse event callback and returns its listener.
fn add_mouse_event<T>(target:&EventTarget, name:&str, closure: T) -> Result<MouseEventListener>
where T : Fn(MouseEvent) + 'static {
    let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
    let callback : &Function = closure.as_ref().unchecked_ref();
    add_event_listener_with_callback(target, name, callback)?;
    Ok(closure)
}

/// Removes mouse event listener.
fn remove_mouse_event(target:&EventTarget, name:&str, listener:MouseEventListener) -> Result<()> {
    let callback : &Function = listener.as_ref().unchecked_ref();
    remove_event_listener_with_callback(target, name, callback)
}

/// Adds wheel event callback and returns its listener.
fn add_wheel_event<T>(target:&EventTarget, closure: T) -> Result<WheelEventListener>
where T : Fn(WheelEvent) + 'static {
    let closure     = Closure::wrap(Box::new(closure) as Box<dyn Fn(WheelEvent)>);
    let callback    = closure.as_ref().unchecked_ref();
    let mut options = AddEventListenerOptions::new();
    options.passive(true);
    match target.add_event_listener_with_callback_and_add_event_listener_options
    ("wheel", callback, &options) {
        Ok(_)  => Ok(closure),
        Err(_) => Err(Error::FailedToAddEventListener)
    }
}

/// Removes wheel event listener.
fn remove_wheel_event(target:&EventTarget, listener:WheelEventListener) -> Result<()> {
    let callback = listener.as_ref().unchecked_ref();
    remove_event_listener_with_callback(target, "wheel", callback)
}