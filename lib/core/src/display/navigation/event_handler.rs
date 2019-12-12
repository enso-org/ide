// FIXME: NEEDS MAJOR REFACTORING!

use crate::system::web::document;
use crate::system::web::dyn_into;
use crate::system::web::ignore_context_menu;

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

pub enum Event {
    Start,
    Zoom(ZoomEvent),
    Pan(PanEvent),
    None
}

#[derive(Clone)]
struct MouseStart {
    button   : i16,
    position : Vector2<f32>
}

struct MouseMove {
    start    : Option<MouseStart>,
    previous : Vector2<f32>,
    current  : Vector2<f32>
}

struct MouseWheel {
    position : Vector2<f32>,
    delta_y  : f32
}

enum MouseEventX {
    Start(MouseStart),
    Move(MouseMove),
    Wheel(MouseWheel),
    None
}

impl MouseEventX {
    pub fn consume(&mut self) -> Self {
        match self {
            MouseEventX::Start(start) => MouseEventX::Start(start.clone()),
            MouseEventX::Move(event)  => {
                let start       = event.start.clone();
                let previous    = event.previous;
                let current     = event.current;
                let mouse_move  = MouseMove { start, previous, current };
                event.previous  = event.current;
                MouseEventX::Move(mouse_move)
            },
            MouseEventX::Wheel(event)  => {
                let position    = event.position;
                let delta_y     = event.delta_y;
                let mouse_wheel = MouseWheel { position, delta_y };
                event.delta_y   = 0.0;
                MouseEventX::Wheel(mouse_wheel)
            },
            MouseEventX::None          => MouseEventX::None
        }
    }
}

// ====================
// === EventHandler ===
// ====================

const MMB : i16 = 1;
const RMB : i16 = 2;

pub struct EventHandler {
    data : RefCell<MouseEventX>
}

fn add_mouse_event
    <T>(target:&EventTarget, name:&str, t : T) -> Closure<dyn Fn(MouseEvent)>
    where T : Fn(MouseEvent) + 'static {
    let closure = Closure::wrap(Box::new(t) as Box<dyn Fn(MouseEvent)>);
    let callback : &Function = closure.as_ref().unchecked_ref();
    target.add_event_listener_with_callback(name, callback).unwrap();
    closure
}

fn add_wheel_event
<T>(target:&EventTarget, t : T) -> Closure<dyn Fn(WheelEvent)>
    where T : Fn(WheelEvent) + 'static {
    let closure = Closure::wrap(Box::new(t) as Box<dyn Fn(WheelEvent)>);
    let callback : &Function = closure.as_ref().unchecked_ref();
    target.add_event_listener_with_callback("wheel", callback).unwrap();
    closure
}

impl EventHandler {
    pub fn new(dom:&DOMContainer) -> Rc<Self> {
        let data = RefCell::new(MouseEventX::None);
        let panning = Rc::new(EventHandler { data });

        let document : EventTarget = dyn_into(document().unwrap()).unwrap();
        let target   : EventTarget = dyn_into(dom.dom.clone()).unwrap();

        ignore_context_menu(&target).forget();

        let panning_clone = panning.clone();
        let closure = move |event:MouseEvent| {
            if event.button() == MMB || event.button() == RMB {
                let mut panning = panning_clone.data.borrow_mut();
                let button      = event.button();
                let position    = Vector2::new(event.x() as f32, event.y() as f32);
                let start       = MouseStart { button, position };
                *panning        = MouseEventX::Start(start);
            }
        };
        add_mouse_event(&target, "mousedown", closure).forget();

        let panning_clone = panning.clone();
        let closure = move |event:MouseEvent| {
            let panning : &mut MouseEventX = &mut panning_clone.data.borrow_mut();
            let x                          = event.x() as f32;
            let y                          = event.y() as f32;
            let current                    = Vector2::new(x, y);
            match panning {
                MouseEventX::Start(mouse_start) => {
                    let previous = mouse_start.position;

                    let start = Some(mouse_start.clone());
                    let mouse_move = MouseMove { start, previous, current };
                    *panning = MouseEventX::Move(mouse_move);
                },
                MouseEventX::Move(mouse_move) => {
                    mouse_move.current = current;
                },
                MouseEventX::Wheel(mouse_wheel) => {
                    mouse_wheel.position = current;
                },
                _ => {
                    let start      = None;
                    let previous   = current;
                    let mouse_move = MouseMove { start, previous, current };
                    *panning       = MouseEventX::Move(mouse_move);
                },
            }
        };
        add_mouse_event(&target, "mousemove", closure).forget();

        let panning_clone = panning.clone();
        let closure = move |_:MouseEvent| {
            let mut panning = panning_clone.data.borrow_mut();
            *panning = MouseEventX::None;
        };
        add_mouse_event(&target, "mouseup", closure.clone()).forget();
        add_mouse_event(&document, "mouseleave", closure).forget();

        let panning_clone = panning.clone();
        let closure = move |event:WheelEvent| {
            let panning : &mut MouseEventX = &mut panning_clone.data.borrow_mut();
            let delta_y                    = event.delta_y() as f32;
            match panning {
                MouseEventX::Start(mouse_start) => {
                    let position    = mouse_start.position;
                    let mouse_wheel = MouseWheel { position, delta_y };
                    *panning        = MouseEventX::Wheel(mouse_wheel);
                },
                MouseEventX::Move(mouse_move) => {
                    let position    = mouse_move.current;
                    let mouse_wheel = MouseWheel { position, delta_y };
                    *panning        = MouseEventX::Wheel(mouse_wheel);
                },
                MouseEventX::Wheel(mouse_wheel) => {
                    mouse_wheel.delta_y = delta_y;
                },
                MouseEventX::None => ()
            }
        };
        add_wheel_event(&target, closure).forget();

        panning
    }

    pub fn poll(&self) -> Event {
        let   event : &mut MouseEventX = &mut self.data.borrow_mut();
        match event.consume() {
            MouseEventX::Start(_)         => Event::Start,
            MouseEventX::Move(mouse_move) => {
                if let Some(start) = &mouse_move.start {
                    let event = match start.button {
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
                    };
                    event
                } else {
                    Event::None
                }
            },
            MouseEventX::Wheel(mouse_wheel) => {
                let focus  = mouse_wheel.position;
                let amount = mouse_wheel.delta_y;
                Event::Zoom(ZoomEvent { focus, amount })
            }
            _ => Event::None
        }
    }
}