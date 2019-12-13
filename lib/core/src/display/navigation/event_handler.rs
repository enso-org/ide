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
    Pan(PanEvent),
    None
}

// ==================
// === MouseStart ===
// ==================

/// MouseStart event.
#[derive(Clone)]
struct MouseStart {
    button   : i16,
    position : Vector2<f32>
}

// =================
// === MouseMove ===
// =================

/// MouseMove event.
struct MouseMove {
    start    : Option<MouseStart>,
    previous : Vector2<f32>,
    current  : Vector2<f32>
}

// ==================
// === MouseWheel ===
// ==================

/// MouseWheel event with focus position.
struct MouseWheel {
    position : Vector2<f32>,
    delta_y  : f32
}

// ==========================
// === InternalMouseEvent ===
// ==========================

/// event_handler's InternalMouseEvents enumeration.
enum InternalMouseEvent {
    Start(MouseStart),
    Move(MouseMove),
    Wheel(MouseWheel),
    None
}

impl InternalMouseEvent {
    /// To consume the event to make sure we don't process it twice.
    pub fn consume(&mut self) -> Self {
        match self {
            InternalMouseEvent::Start(event) => Self::consume_start(event),
            InternalMouseEvent::Move (event) => Self::consume_move (event),
            InternalMouseEvent::Wheel(event) => Self::consume_wheel(event),
            InternalMouseEvent::None         => InternalMouseEvent::None
        }
    }

    fn consume_start(event:&mut MouseStart) -> Self {
        // We don't need to modify Start, so we just clone it.
        InternalMouseEvent::Start(event.clone())
    }

    fn consume_move(event:&mut MouseMove) -> Self {
        let start       = event.start.clone();
        let previous    = event.previous;
        let current     = event.current;
        let mouse_move  = MouseMove { start, previous, current };
        // To consume move, we set event.previous = event.current.
        event.previous  = event.current;
        InternalMouseEvent::Move(mouse_move)
    }

    fn consume_wheel(event:&mut MouseWheel) -> Self {
        let position    = event.position;
        let delta_y     = event.delta_y;
        let mouse_wheel = MouseWheel { position, delta_y };
        // To consume wheel, we set delta_y = 0.
        event.delta_y   = 0.0;
        InternalMouseEvent::Wheel(mouse_wheel)
    }
}

// ====================
// === EventHandler ===
// ====================

const MMB : i16 = 1;
const RMB : i16 = 2;

/// Struct used to handle mouse events such as MouseDown, MouseMove, MouseUp,
/// MouseLeave and MouseWheel.
pub struct EventHandler {
    data : RefCell<InternalMouseEvent>
}

impl EventHandler {
    pub fn new(dom:&DOMContainer) -> Result<Rc<Self>> {
        let data     = RefCell::new(InternalMouseEvent::None);
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
            if event.button() == MMB || event.button() == RMB {
                let mut ehandler = ehandler.data.borrow_mut();
                let button       = event.button();
                let x            = event.x() as f32;
                let y            = event.y() as f32;
                let position     = Vector2::new(x, y);
                let start        = MouseStart { button, position };
                *ehandler        = InternalMouseEvent::Start(start);
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
            let ime     = &mut ehandler.data.borrow_mut();
            let ime     = ime as &mut InternalMouseEvent;
            let x       = event.x() as f32;
            let y       = event.y() as f32;
            let current = Vector2::new(x, y);
            match ime {
                InternalMouseEvent::Start(mouse_start) => {
                    let previous = mouse_start.position;

                    let start      = Some(mouse_start.clone());
                    let mouse_move = MouseMove { start, previous, current };
                    *ime = InternalMouseEvent::Move(mouse_move);
                },
                InternalMouseEvent::Move(mouse_move) => {
                    mouse_move.current = current;
                },
                InternalMouseEvent::Wheel(mouse_wheel) => {
                    mouse_wheel.position = current;
                },
                _ => {
                    let start      = None;
                    let previous   = current;
                    let mouse_move = MouseMove { start, previous, current };
                    *ime = InternalMouseEvent::Move(mouse_move);
                },
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
            *ime        = InternalMouseEvent::None;
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
            let ime     = &mut ehandler.data.borrow_mut();
            let ime     = ime as &mut InternalMouseEvent;
            let delta_y = event.delta_y() as f32;
            match ime {
                InternalMouseEvent::Start(mouse_start) => {
                    let position    = mouse_start.position;
                    let mouse_wheel = MouseWheel { position, delta_y };
                    *ime            = InternalMouseEvent::Wheel(mouse_wheel);
                },
                InternalMouseEvent::Move(mouse_move) => {
                    let position    = mouse_move.current;
                    let mouse_wheel = MouseWheel { position, delta_y };
                    *ime            = InternalMouseEvent::Wheel(mouse_wheel);
                },
                InternalMouseEvent::Wheel(mouse_wheel) => {
                    mouse_wheel.delta_y = delta_y;
                },
                InternalMouseEvent::None => ()
            }
        };
        add_wheel_event(&target, closure)?.forget();
        Ok(())
    }

    /// Checks if EventHandler has an event.
    pub fn poll(&self) -> Event {
        let   event : &mut InternalMouseEvent = &mut self.data.borrow_mut();
        match event.consume() {
            InternalMouseEvent::Start(_)         => Event::Start,
            InternalMouseEvent::Move(mouse_move) => {
                if let Some(start) = &mouse_move.start {
                    match start.button {
                        MMB => {
                            let mut movement   =  mouse_move.current;
                                    movement  -=  mouse_move.previous;
                                    movement.x = -movement.x;
                            Event::Pan(PanEvent { movement })
                        },
                        RMB => {
                            let focus  = start.position;
                            let mut amount  = mouse_move.current.y;
                                    amount -= mouse_move.previous.y;
                            Event::Zoom(ZoomEvent { focus, amount })
                        },
                        _ => Event::None
                    }
                } else {
                    Event::None
                }
            },
            InternalMouseEvent::Wheel(mouse_wheel) => {
                let focus  = mouse_wheel.position;
                let amount = mouse_wheel.delta_y;
                Event::Zoom(ZoomEvent { focus, amount })
            }
            _ => Event::None
        }
    }
}

// =============
// === Utils ===
// =============

/// Adds mouse event callback to target.
fn add_mouse_event
<T>(target:&EventTarget, name:&str, t : T)
    -> Result<Closure<dyn Fn(MouseEvent)>>
    where T : Fn(MouseEvent) + 'static {
    let closure = Closure::wrap(Box::new(t) as Box<dyn Fn(MouseEvent)>);
    let callback : &Function = closure.as_ref().unchecked_ref();
    match target.add_event_listener_with_callback(name, callback) {
        Ok(_) => Ok(closure),
        Err(_) => Err(Error::AddEventListenerFail)
    }
}

/// Adds wheel event callback to target.
fn add_wheel_event
<T>(target:&EventTarget, t : T) -> Result<Closure<dyn Fn(WheelEvent)>>
    where T : Fn(WheelEvent) + 'static {
    let closure = Closure::wrap(Box::new(t) as Box<dyn Fn(WheelEvent)>);
    let callback : &Function = closure.as_ref().unchecked_ref();
    match target.add_event_listener_with_callback("wheel", callback) {
        Ok(_)  => Ok(closure),
        Err(_) => Err(Error::AddEventListenerFail)
    }
}