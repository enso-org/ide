// FIXME: NEEDS MAJOR REFACTORING!

use crate::system::web::document;
use crate::system::web::dyn_into;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Function;
use web_sys::MouseEvent;
use web_sys::EventTarget;
use nalgebra::Vector2;
use nalgebra::Vector3;
use std::rc::Rc;
use std::cell::RefCell;
use crate::display::rendering::DOMContainer;

use crate::system::web::console_log;

// ===================
// === PanningData ===
// ===================

struct PanningData {
    begin    : bool,
    depth    : bool,
    start    : Vector2<f32>,
    prev     : Vector2<f32>,
    end      : Vector2<f32>
}

impl PanningData {
    fn start(depth:bool, start : Vector2<f32>) -> Self {
        let prev = start;
        let end = start;
        let begin = true;
        Self { begin, start, prev, end, depth }
    }
}

pub struct Event {
    pub begin    : bool,
    pub start    : Vector2<f32>,
    pub movement : Vector3<f32>
}

// ===============
// === Panning ===
// ===============

const LMB : i16 = 0;
const MMB : i16 = 1; // MMB for middle mouse button? :P
const RMB : i16 = 2;

pub struct Panning {
    data : RefCell<Option<PanningData>>
}

impl Panning {
    pub fn new(dom:&DOMContainer) -> Rc<Self> {
        let data = RefCell::new(None);
        let panning = Rc::new(Panning { data });

        let document : EventTarget = dyn_into(document().unwrap()).unwrap();
        let target : EventTarget = dyn_into(dom.dom.clone()).unwrap();

        let closure = move |event:MouseEvent| {
            if event.button() == RMB {
                event.prevent_default();
            }
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("contextmenu", callback);
        closure.forget();

        let panning_clone = panning.clone();
        let closure = move |event:MouseEvent| {
            if event.button() == MMB || event.button() == RMB {
                let mut panning = panning_clone.data.borrow_mut();
                let start = Vector2::new(event.x() as f32, event.y() as f32);
                *panning = Some(PanningData::start(event.button() == RMB, start));
            }
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("mousedown", callback);
        closure.forget();

        let panning_clone = panning.clone();
        let closure = move |event:MouseEvent| {
            let mut panning = panning_clone.data.borrow_mut();
            let panning     = panning.as_mut();
            if let Some(panning) = panning {
                panning.begin = false;
                panning.end = Vector2::new(event.x() as f32, event.y() as f32);
            }
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        document.add_event_listener_with_callback("mousemove", callback);
        closure.forget();

        let panning_clone = panning.clone();
        let closure = move |_:MouseEvent| {
            let mut panning = panning_clone.data.borrow_mut();
            *panning = None;
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        document.add_event_listener_with_callback("mouseup", callback);
        closure.forget();

        let panning_clone = panning.clone();
        let closure = move |_:MouseEvent| {
            let mut panning = panning_clone.data.borrow_mut();
            *panning = None;
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        document.add_event_listener_with_callback("mouseleave", callback);
        closure.forget();

        panning
    }

    pub fn consume(&self) -> Option<(Event)> {
        let mut panning = self.data.borrow_mut();
        let     panning = panning.as_mut();
        if let Some(panning) = panning {
            let res = panning.end - panning.prev;
            panning.prev = panning.end;
            let vector = if panning.depth {
                Vector3::new(0.0, 0.0, res.y)
            } else {
                Vector3::new(-res.x, res.y, 0.0)
            };
            let movement = vector;
            let begin = panning.begin;
            let start = panning.start;
            Some(Event { start, movement, begin })
        } else {
            None
        }
    }
}