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
use std::any::Any;


// FIXME: We need to use one style headers everywhere. Please change these to short ones and if you think longer are better, lets talk with the whole IDE team on Discord first and if we all agree they are better, we will change them in the whole codebase with some kind of regexp.
// =================================================================================================
// === EventListener ===============================================================================
// =================================================================================================

/// This struct keeps the register of the event listener and unregisters it when it's dropped.
pub struct EventListener { // FIXME: spacing + WOHOOOO! Not parametrized with `T` anymore!
    target  : EventTarget,
    name    : String,
    closure : Function // FIXME: Danilo, I would really love to ask you to go slowly trough all your names and think if they are named semantically or syntactically. I know that this is a closure. It is like naming `int:i32` :P We need to name variables by describing their purpose, not their type / shape.
}

impl EventListener {
    fn new<T:?Sized>(target:EventTarget, name:String, closure:Closure<T>) -> Self {
        let closure: &Function = closure.as_ref().unchecked_ref();
        let closure = closure.clone();
        Self { target, name, closure }
    }
}

impl Drop for EventListener {
    fn drop(&mut self) {
        remove_event_listener_with_callback(&self.target,&self.name,&self.closure).ok();
    }
}
// FIXME: Spacing
// =================================================================================================
// === MouseButton =================================================================================
// =================================================================================================

/// An enumeration representing the mouse buttons.
pub enum MouseButton {
    LEFT,
    MIDDLE,
    RIGHT
}



// =================================================================================================
// === MouseClickEvent =============================================================================
// =================================================================================================

// FIXME: Maybe it's ok, but why you don't define it as FnMut?
pub trait FnMouseClick = Fn(MouseClickEvent) + 'static; // TODO: To make the name more semantical, lets rename it to `MouseClickCallback` please :)

/// A struct storing information about mouse down and mouse up events.
pub struct MouseClickEvent {
    pub position : Vector2<f32>,
    pub button   : MouseButton
}

impl MouseClickEvent {
    fn from(event:MouseEvent, data:&Rc<MouseManagerData>) -> Self {
        let position  = Vector2::new(event.x() as f32, event.y() as f32);
        let position  = position - data.dom().position();
        let button    = match event.button() {
            LEFT_MOUSE_BUTTON      => MouseButton::LEFT,
            MIDDLE_MOUSE_BUTTON    => MouseButton::MIDDLE,
            RIGHT_MOUSE_BUTTON | _ => MouseButton::RIGHT // TODO: Please, do not revert to right mouse button. You can drop the events, but do not convert other buttons to RMB!
        };
        Self { position, button }
    }
}



// =================================================================================================
// === MousePositionEvent ==========================================================================
// =================================================================================================

pub trait FnMousePosition = Fn(MousePositionEvent)  + 'static;

/// A struct storing information about mouse move, mouse enter and mouse leave events.
pub struct MousePositionEvent {
    pub previous_position : Vector2<f32>,
    pub position          : Vector2<f32>
}

impl MousePositionEvent {
    fn from(event:MouseEvent, data:&Rc<MouseManagerData>) -> Self {
        let position  = Vector2::new(event.x() as f32, event.y() as f32); // FIXME: spacing
        let position  = position - data.dom().position();
        let previous_position = match data.mouse_position() {
            Some(position) => position,
            None           => position
        };
        data.set_mouse_position(Some(position));
        Self { previous_position, position }
    }
}



// =================================================================================================
// === TouchPadEventDetector =======================================================================
// =================================================================================================


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



// =================================================================================================
// === MouseWheelEvent =============================================================================
// =================================================================================================

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



// =================================================================================================
// === MouseManagerCell ============================================================================
// =================================================================================================

struct MouseManagerCell { // FIXME: We do not use this naming convention. This sohuld be `MouseManagerData` or something similar. anyway, this is not a cell! Cell means mutable from different places. `RefCell<Something>` is a cell. this is not.
    detector            : TouchPadEventDetector,
    dom                 : DOMContainer,
    mouse_position      : Option<Vector2<f32>>,
    target              : EventTarget,
    stop_mouse_tracking : Option<EventListener> // FIXME: this needs a better name. "stop_mouse_tracking" is like name of a function.
}



// =================================================================================================
// === MouseManagerData ============================================================================
// =================================================================================================

/// A struct used for storing shared MouseManager's mutable data.
struct MouseManagerData {
    cell : RefCell<MouseManagerCell> // FIXME: why MouseManagerData contains `RefCell` only, but not `Rc`, and everywehre it is used it is used as `Rc<MouseManagerData>`?
}

impl MouseManagerData {
    fn new(target:EventTarget, dom:DOMContainer) -> Rc<Self> {
        let detector            = TouchPadEventDetector::new();
        let mouse_position      = None;
        let stop_mouse_tracking = None;
        let cell                = MouseManagerCell {
            detector,
            dom,
            mouse_position,
            target,
            stop_mouse_tracking
        };
        // FIXME: make it like this (below). Its better to break alignment than make 7-line expression :)
        // FIXME: let cell = MouseManagerCell { detector,dom,mouse_position,target,stop_mouse_tracking};
        let cell = RefCell::new(cell);
        Rc::new(Self { cell })
    }
}


// === Setters =====================================================================================

impl MouseManagerData {
    fn set_mouse_position(&self, position:Option<Vector2<f32>>) {
        self.cell.borrow_mut().mouse_position = position
    }

    fn set_stop_mouse_tracking(&self, listener:Option<EventListener>) {
        self.cell.borrow_mut().stop_mouse_tracking = listener;
    }

    fn mod_detector<F:FnOnce(&mut TouchPadEventDetector)>(&self, f:F) {
        (f)(&mut self.cell.borrow_mut().detector)
    }
}


// === Getters =====================================================================================

impl MouseManagerData {
    fn target(&self) -> EventTarget { self.cell.borrow().target.clone() }

    fn mouse_position(&self) -> Option<Vector2<f32>> { self.cell.borrow().mouse_position }

    fn dom(&self) -> DOMContainer { self.cell.borrow().dom.clone() }
}



// =================================================================================================
// === MouseManager ================================================================================
// =================================================================================================

/// This structs manages mouse events in a specified DOM object.
pub struct MouseManager {
    data                : Rc<MouseManagerData> // FIXME: spacing
}

const   LEFT_MOUSE_BUTTON: i16 = 0;
const MIDDLE_MOUSE_BUTTON: i16 = 1;
const  RIGHT_MOUSE_BUTTON: i16 = 2;

impl MouseManager {
    pub fn new(dom:&DOMContainer) -> Result<Self> {
        let target              = dyn_into::<_, EventTarget>(dom.dom.clone())?;
        let dom                 = dom.clone();
        let data                = MouseManagerData::new(target,dom);
        let mut mouse_manager   = Self { data };
        mouse_manager.stop_tracking_mouse_when_it_leaves_dom()?;
        Ok(mouse_manager)
    }

    /// Sets context menu state to enabled or disabled.
    pub fn disable_context_menu(&mut self) -> Result<EventListener> {
        let listener = ignore_context_menu(&self.data.target())?;
        Ok(EventListener::new(self.data.target(), "contextmenu".to_string(), listener)) // FIXME: this is ugly a little bit. Please make the `new` function in such way that it accepts both `String` as well as `&str`. To make nice API, the conversion should always be done on definition side, not use-side! :)
    }

    // FIXME: Can we generate the below things with macro_rules to throw away the boilerplate? Or would it be impractical?
    /// Adds mouse down event callback and returns its listener object.
    pub fn add_mouse_down_callback<F:FnMouseClick>(&mut self, f:F) -> Result<EventListener> {
        let data = Rc::downgrade(&self.data);
        let closure = move |event:MouseEvent| {
            if let Some(data) = data.upgrade() {
                f(MouseClickEvent::from(event, &data));
            }
        };
        add_mouse_event(&self.data.target(), "mousedown", closure)
    }

    /// Adds mouse up event callback and returns its listener object.
    pub fn add_mouse_up_callback<F:FnMouseClick>(&mut self, f:F) -> Result<EventListener> {
        let data = Rc::downgrade(&self.data);
        let closure = move |event:MouseEvent| {
            if let Some(data) = data.upgrade() {
                f(MouseClickEvent::from(event, &data));
            }
        };
        add_mouse_event(&self.data.target(), "mouseup", closure)
    }

    /// Adds mouse move event callback and returns its listener object.
    pub fn add_mouse_move_callback
    <F:FnMousePosition>(&mut self, f:F) -> Result<EventListener> {
        let data = Rc::downgrade(&self.data);
        let closure = move |event:MouseEvent| {
            if let Some(data) = data.upgrade() {
                f(MousePositionEvent::from(event, &data));
            }
        };
        add_mouse_event(&self.data.target(), "mousemove", closure)
    }

    /// Adds mouse leave event callback and returns its listener object.
    pub fn add_mouse_leave_callback
    <F:FnMousePosition>(&mut self, f:F) -> Result<EventListener> {
        let data = Rc::downgrade(&self.data);
        let closure = move |event:MouseEvent| {
            if let Some(data) = data.upgrade() {
                f(MousePositionEvent::from(event, &data));
            }
        };
        add_mouse_event(&self.data.target(), "mouseleave", closure)
    }

    /// Adds MouseWheel event callback and returns its listener object.
    pub fn add_mouse_wheel_callback
    <F:FnMouseWheel>(&mut self, mut f:F) -> Result<EventListener> {
        let data = Rc::downgrade(&self.data);
        let closure = move |event:WheelEvent| {
            if let Some(data) = data.upgrade() {
                data.mod_detector(|mut detector| {
                    f(MouseWheelEvent::from(event, &mut detector));
                });
            }
        };
        add_wheel_event(&self.data.target(), closure)
    }

    fn stop_tracking_mouse_when_it_leaves_dom(&mut self) -> Result<()> {
        let data     = Rc::downgrade(&self.data);
        let closure  = move |_| {
            if let Some(data) = data.upgrade() {
                data.set_mouse_position(None);
            }
        };
        let listener = add_mouse_event(&self.data.target(), "mouseleave", closure)?;
        self.data.set_stop_mouse_tracking(Some(listener));
        Ok(())
    }
}



// =================================================================================================
// === Utils =======================================================================================
// =================================================================================================

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
fn add_mouse_event<T>(target:&EventTarget, name:&str, closure: T) -> Result<EventListener>
where T : Fn(MouseEvent) + 'static {
    let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
    let callback : &Function = closure.as_ref().unchecked_ref();
    add_event_listener_with_callback(target, name, callback)?;
    Ok(EventListener::new(target.clone(), name.to_string(), closure))
}

/// Adds wheel event callback and returns its listener.
fn add_wheel_event<T>(target:&EventTarget, closure: T) -> Result<EventListener>
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
            let listener = EventListener::new(target, name, closure);
            Ok(listener)
        },
        Err(_) => Err(Error::FailedToAddEventListener)
    }
}