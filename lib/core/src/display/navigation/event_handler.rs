// FIXME: NEEDS MAJOR REFACTORING!

use crate::system::web::document;
use crate::system::web::dyn_into;
use crate::system::web::ignore_context_menu;
use crate::system::web::EventListeningResult as Result;
use crate::system::web::EventListeningError as Error;

use crate::display::rendering::DOMContainer;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Function;
use web_sys::MouseEvent;
use web_sys::WheelEvent;
use web_sys::EventTarget;
use web_sys::AddEventListenerOptions;
use nalgebra::Vector2;
use std::rc::Rc;
use std::cell::RefCell;

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
    Start,
    Zoom(ZoomEvent),
    Pan(PanEvent)
}

// ==================
// === MouseStart ===
// ==================

/// MouseStart event.
#[derive(Clone)]
struct MouseStartEvent {
    button   : i16,
    position : Vector2<f32>
}

// =================
// === MouseMove ===
// =================

/// MouseMove event.
struct MouseMoveEvent {
    start    : Option<MouseStartEvent>,
    previous : Vector2<f32>,
    current  : Vector2<f32>
}

// ==================
// === MouseWheel ===
// ==================

/// MouseWheel event with focus position.
struct MouseWheelEvent {
    position : Vector2<f32>,
    delta_y  : f32
}

// ==========================
// === InternalMouseEvent ===
// ==========================

/// Enumeration to represent mouse events.
enum InternalMouseEvent {
    Start(MouseStartEvent),
    Move(MouseMoveEvent),
    Wheel(MouseWheelEvent)
}

impl InternalMouseEvent {
    /// To consume the event to make sure we don't process it twice.
    pub fn consume(&mut self) -> Self {
        match self {
            InternalMouseEvent::Start(event) => Self::consume_start(event),
            InternalMouseEvent::Move (event) => Self::consume_move (event),
            InternalMouseEvent::Wheel(event) => Self::consume_wheel(event)
        }
    }

    fn consume_start(event:&mut MouseStartEvent) -> Self {
        // We don't need to modify Start, so we just clone it.
        InternalMouseEvent::Start(event.clone())
    }

    fn consume_move(event:&mut MouseMoveEvent) -> Self {
        let start       = event.start.clone();
        let previous    = event.previous;
        let current     = event.current;
        let mouse_move  = MouseMoveEvent { start, previous, current };
        // To consume move, we set event.previous = event.current.
        event.previous  = event.current;
        InternalMouseEvent::Move(mouse_move)
    }

    fn consume_wheel(event:&mut MouseWheelEvent) -> Self {
        let position    = event.position;
        let delta_y     = event.delta_y;
        let mouse_wheel = MouseWheelEvent { position, delta_y };
        // To consume wheel, we set delta_y = 0.
        event.delta_y   = 0.0;
        InternalMouseEvent::Wheel(mouse_wheel)
    }
}

// ====================
// === EventHandler ===
// ====================

const MIDDLE_MOUSE_BUTTON: i16 = 1;
const  RIGHT_MOUSE_BUTTON: i16 = 2;

/// Struct used to handle mouse events such as MouseDown, MouseMove, MouseUp,
/// MouseLeave and MouseWheel.
pub struct EventHandler {
    data : RefCell<Option<InternalMouseEvent>>
}

impl EventHandler {
    pub fn new(dom:&DOMContainer) -> Result<Rc<Self>> {
        let data     = RefCell::new(None);
        let ehandler = Rc::new(Self { data });

        let document : EventTarget = dyn_into(document().unwrap()).unwrap();
        let target   : EventTarget = dyn_into(dom.dom.clone()).unwrap();

        ehandler.initialize_events(&target, &document)?;
        Ok(ehandler)
    }

    /// Initialize mouse events.
    fn initialize_events
    (self:&Rc<Self>, target:&EventTarget, document:&EventTarget) -> Result<()> {
        ignore_context_menu(&target)?.forget();
        self.initialize_mouse_start(&target)?;
        self.initialize_mouse_move(&target)?;
        self.initialize_mouse_end(&target, &document)?;
        self.initialize_wheel_event(&target)?;
        Ok(())
    }

    /// Initialize MouseDown event callback.
    fn initialize_mouse_start
    (self:&Rc<Self>, target:&EventTarget) -> Result<()> {
        let ehandler = self.clone();
        let closure = move |event:MouseEvent| {
            if event.button() == MIDDLE_MOUSE_BUTTON || event.button() == RIGHT_MOUSE_BUTTON {
                let mut ehandler = ehandler.data.borrow_mut();
                let button       = event.button();
                let x            = event.x() as f32;
                let y            = event.y() as f32;
                let position     = Vector2::new(x, y);
                let start        = MouseStartEvent { button, position };
                *ehandler        = Some(InternalMouseEvent::Start(start));
            }
        };
        add_mouse_event(&target, "mousedown", closure)?.forget();
        Ok(())
    }

    /// Initialize MouseMove event callback.
    fn initialize_mouse_move
    (self:&Rc<Self>, target:&EventTarget) -> Result<()> {
        let ehandler = self.clone();
        let closure = move |event:MouseEvent| {
            let mut data = ehandler.data.borrow_mut();
            let mut ime = data.as_mut();
            let x = event.x() as f32;
            let y = event.y() as f32;
            let current = Vector2::new(x, y);
            if let Some(ime) = &mut ime {
                match ime {
                    InternalMouseEvent::Start(mouse_start) => {
                        let previous = mouse_start.position;

                        let start = Some(mouse_start.clone());
                        let mouse_move = MouseMoveEvent { start, previous, current };
                        **ime = InternalMouseEvent::Move(mouse_move);
                    },
                    InternalMouseEvent::Move(mouse_move) => {
                        mouse_move.current = current;
                    },
                    InternalMouseEvent::Wheel(mouse_wheel) => {
                        mouse_wheel.position = current;
                    },
                }
            } else {
                let start = None;
                let previous = current;
                let mouse_move = MouseMoveEvent { start, previous, current };
                *data = Some(InternalMouseEvent::Move(mouse_move));
            }
        };
        add_mouse_event(&target, "mousemove", closure)?.forget();
        Ok(())
    }

    /// Initialize MouseUp and MouseLeave events callbacks.
    fn initialize_mouse_end
    (self:&Rc<Self>, target:&EventTarget, document:&EventTarget) -> Result<()>{
        let ehandler = self.clone();
        let closure = move |_:MouseEvent| {
            let mut ime = ehandler.data.borrow_mut();
            *ime        = None;
        };
        add_mouse_event(&target  , "mouseup"   , closure.clone())?.forget();
        add_mouse_event(&document, "mouseleave", closure        )?.forget();
        Ok(())
    }

    /// Initialize wheel event callback.
    fn initialize_wheel_event
    (self:&Rc<Self>, target:&EventTarget) -> Result<()>{
        let ehandler = self.clone();
        let closure = move |event:WheelEvent| {
            let mut ime = ehandler.data.borrow_mut();
            let mut ime = ime.as_mut();
            if let Some(ime) = &mut ime {
                let delta_y = event.delta_y() as f32;
                match ime {
                    InternalMouseEvent::Start(mouse_start) => {
                        let position = mouse_start.position;
                        let mouse_wheel = MouseWheelEvent { position, delta_y };
                        let mouse_event = InternalMouseEvent::Wheel(mouse_wheel);
                        **ime = mouse_event;
                    },
                    InternalMouseEvent::Move(mouse_move) => {
                        let position = mouse_move.current;
                        let mouse_wheel = MouseWheelEvent { position, delta_y };
                        let mouse_event = InternalMouseEvent::Wheel(mouse_wheel);
                        **ime = mouse_event;
                    },
                    InternalMouseEvent::Wheel(mouse_wheel) => {
                        mouse_wheel.delta_y = delta_y;
                    }
                }
            }
        };
        add_wheel_event(&target, closure)?.forget();
        Ok(())
    }

    /// Checks if EventHandler has an event.
    pub fn poll(&self) -> Option<Event> {
        let mut event = self.data.borrow_mut();
        let event = event.as_mut();
        if let Some(event) = event {
            match event.consume() {
                InternalMouseEvent::Start(_) => Some(Event::Start),
                InternalMouseEvent::Move(mouse_move) => {
                    if let Some(start) = &mouse_move.start {
                        match start.button {
                            MIDDLE_MOUSE_BUTTON => {
                                let mut movement = mouse_move.current;
                                movement -= mouse_move.previous;
                                movement.x = -movement.x;
                                Some(Event::Pan(PanEvent { movement }))
                            },
                            RIGHT_MOUSE_BUTTON => {
                                let focus = start.position;
                                let mut amount = mouse_move.current.y;
                                amount -= mouse_move.previous.y;
                                Some(Event::Zoom(ZoomEvent { focus, amount }))
                            },
                            _ => None
                        }
                    } else {
                        None
                    }
                },
                InternalMouseEvent::Wheel(mouse_wheel) => {
                    let focus = mouse_wheel.position;
                    let amount = mouse_wheel.delta_y;
                    Some(Event::Zoom(ZoomEvent { focus, amount }))
                }
            }
        } else {
            None
        }
    }
}

// =============
// === Utils ===
// =============

// FIXME: documentation, formatting
/// Adds a generated wasm_bindgen::prelude::Closure from the `closure` param
/// to the `EventTarget` and returns it.
fn add_mouse_event
<T>(target:&EventTarget, name:&str, closure: T)
-> Result<Closure<dyn Fn(MouseEvent)>>
where T : Fn(MouseEvent) + 'static {
    let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
    let callback : &Function = closure.as_ref().unchecked_ref();
    match target.add_event_listener_with_callback(name, callback) {
        Ok(_) => Ok(closure),
        Err(_) => Err(Error::AddEventListenerFail)
    }
}

// FIXME: documentation, formatting
/// Adds a generated wasm_bindgen::prelude::Closure from the `closure` param
/// to the `EventTarget` and returns it.
fn add_wheel_event
<T>(target:&EventTarget, closure: T) -> Result<Closure<dyn Fn(WheelEvent)>>
where T : Fn(WheelEvent) + 'static {
    let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(WheelEvent)>);
    let callback : &Function = closure.as_ref().unchecked_ref();
    let mut options = AddEventListenerOptions::new();
    options.passive(true);
    match target.add_event_listener_with_callback_and_add_event_listener_options
    ("wheel", callback, &options) {
        Ok(_)  => Ok(closure),
        Err(_) => Err(Error::AddEventListenerFail)
    }
}