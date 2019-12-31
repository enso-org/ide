use crate::display::render::css3d::DOMContainer;
use crate::system::web::dyn_into;
use crate::system::web::Result;
use crate::system::web::Error;
use crate::system::web::ignore_context_menu;
use crate::system::web::get_performance;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;
use web_sys::WheelEvent;
use web_sys::EventTarget;
use web_sys::AddEventListenerOptions;
use web_sys::Performance;
use js_sys::Function;
use nalgebra::Vector2;
use std::rc::Rc;
use std::cell::RefCell;


// =====================
// === EventListener ===
// =====================

/// This struct keeps the register of the event listener and unregisters it when it's dropped.
pub struct EventListener<T : ?Sized> {
    target  : EventTarget,
    name    : String,
    closure : Closure<T>
}

impl<T : ?Sized> EventListener<T> {
    fn new(target:EventTarget, name:String, closure:Closure<T>) -> Self {
        Self { target, name, closure }
    }
}

impl<T : ?Sized> Drop for EventListener<T> {
    fn drop(&mut self) {
        let callback : &Function = self.closure.as_ref().unchecked_ref();
        remove_event_listener_with_callback(&self.target, &self.name, callback).ok();
    }
}

// =============================
// === Mouse Event Listeners ===
// =============================

pub type MouseEventListener = EventListener<dyn Fn(MouseEvent)>;
pub type WheelEventListener = EventListener<dyn FnMut(WheelEvent)>;



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

pub trait FnMouseClick = Fn(MouseClickEvent) + 'static;

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



// ==========================
// === MousePositionevent ===
// ==========================

pub trait FnMousePosition = Fn(MousePositionEvent)  + 'static;

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



// =============================
// === TouchPadEventDetector ===
// =============================


struct TouchPadEventDetector {
    is_touchpad : bool,
    performance : Performance,
    count       : u32,
    start       : f64
}

impl TouchPadEventDetector {
    fn new() -> Self {
        let performance = get_performance().expect("Couldn't get performance");
        let is_touchpad = false;
        let count       = 0;
        let start       = performance.now();
        Self { is_touchpad,performance,count,start }
    }

    fn is_touchpad(&mut self) -> bool {
        let current_time = self.performance.now();

        if self.count == 0 {
            self.start = current_time;
        }

        self.count += 1;

        if current_time - self.start > 100.0 {
            self.is_touchpad = self.count > 5;
            self.count = 0;
        }

        self.is_touchpad
    }
}



// =======================
// === MouseWheelEvent ===
// =======================

pub trait FnMouseWheel = FnMut(MouseWheelEvent) + 'static;

/// A struct storing information about mouse wheel events.
pub struct MouseWheelEvent {
    pub is_touchpad     : bool,
    pub is_ctrl_pressed : bool,
    pub movement_x      : f32,
    pub movement_y      : f32
}

impl MouseWheelEvent {
    fn from(event:WheelEvent, detector:&mut TouchPadEventDetector) -> Self {
        let is_touchpad     = detector.is_touchpad();
        let movement_x      = event.delta_x() as f32;
        let movement_y      = event.delta_y() as f32;
        let is_ctrl_pressed = event.ctrl_key();
        Self { is_touchpad,movement_x,movement_y,is_ctrl_pressed }
    }
}



// ========================
// === MouseManagerData ===
// ========================

/// A struct used for storing shared MouseManager's mutable data.
struct MouseManagerData {
    detector       : TouchPadEventDetector,
    dom            : DOMContainer,
    mouse_position : Option<Vector2<f32>>
}



// ========================
// === ContextMenuState ===
// ========================

/// An enum mainly used for enabling or disabling the Context Menu.
pub enum ContextMenuState {
    Enabled,
    Disabled
}



// ====================
// === MouseManager ===
// ====================

/// This structs manages mouse events in a specified DOM object.
pub struct MouseManager {
    target              : EventTarget,
    data                : Rc<RefCell<MouseManagerData>>,
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
        let detector            = TouchPadEventDetector::new();
        let data                = MouseManagerData { dom,mouse_position,detector };
        let data                = Rc::new(RefCell::new(data));
        let ignore_context_menu = None;
        let stop_mouse_tracking = None;
        let mut mouse_manager   = Self { target, data, ignore_context_menu, stop_mouse_tracking };
        mouse_manager.stop_tracking_mouse_when_it_leaves_dom()?;
        Ok(mouse_manager)
    }

    /// Sets context menu state to enabled or disabled.
    pub fn set_context_menu(&mut self, state: ContextMenuState) -> Result<()> {
        match state {
            ContextMenuState::Enabled => {
                if let Some(_callback) = &self.ignore_context_menu {
                    self.ignore_context_menu = None;
                }
            },
            ContextMenuState::Disabled => {
                if self.ignore_context_menu.is_none() {
                    let listener = ignore_context_menu(&self.target)?;
                    let target = self.target.clone();
                    let name = "contextmenu".to_string();
                    let listener = MouseEventListener::new(target, name, listener);
                    self.ignore_context_menu = Some(listener);
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

    /// Adds mouse up event callback and returns its listener object.
    pub fn add_mouse_up_callback<F:FnMouseClick>(&mut self, f:F) -> Result<MouseEventListener> {
        let data = self.data.clone();
        let closure = move |event:MouseEvent| f(MouseClickEvent::from(event, &data));
        add_mouse_event(&self.target, "mouseup", closure)
    }

    /// Adds mouse move event callback and returns its listener object.
    pub fn add_mouse_move_callback
    <F:FnMousePosition>(&mut self, f:F) -> Result<MouseEventListener> {
        let data = self.data.clone();
        let closure = move |event:MouseEvent| f(MousePositionEvent::from(event, &data));
        add_mouse_event(&self.target, "mousemove", closure)
    }

    /// Adds mouse leave event callback and returns its listener object.
    pub fn add_mouse_leave_callback
    <F:FnMousePosition>(&mut self, f:F) -> Result<MouseEventListener> {
        let data = self.data.clone();
        let closure = move |event:MouseEvent| f(MousePositionEvent::from(event, &data));
        add_mouse_event(&self.target, "mouseleave", closure)
    }

    /// Adds MouseWheel event callback and returns its listener object.
    pub fn add_mouse_wheel_callback
    <F:FnMouseWheel>(&mut self, mut f:F) -> Result<WheelEventListener> {
        let data = self.data.clone();
        let closure = move |event:WheelEvent| {
            f(MouseWheelEvent::from(event, &mut data.borrow_mut().detector));
        };
        add_wheel_event(&self.target, closure)
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
    Ok(MouseEventListener::new(target.clone(), name.to_string(), closure))
}

/// Adds wheel event callback and returns its listener.
fn add_wheel_event<T>(target:&EventTarget, closure: T) -> Result<WheelEventListener>
where T : FnMut(WheelEvent) + 'static {
    let closure     = Closure::wrap(Box::new(closure) as Box<dyn FnMut(WheelEvent)>);
    let callback    = closure.as_ref().unchecked_ref();
    let mut options = AddEventListenerOptions::new();
    options.passive(true);
    match target.add_event_listener_with_callback_and_add_event_listener_options
    ("wheel", callback, &options) {
        Ok(_)  => {
            let target = target.clone();
            let name = "wheel".to_string();
            let listener = WheelEventListener::new(target, name, closure);
            Ok(listener)
        },
        Err(_) => Err(Error::FailedToAddEventListener)
    }
}