// FIXME: NEEDS MAJOR REFACTORING!

use crate::system::web::document;
use crate::system::web::dyn_into;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Function;
use web_sys::MouseEvent;
use web_sys::EventTarget;
use nalgebra::Vector2;
use std::rc::Rc;
use std::cell::RefCell;

// ===================
// === PanningData ===
// ===================

struct PanningData {
    start    : Vector2<f32>,
    end      : Vector2<f32>
}

impl PanningData {
    fn start(start : Vector2<f32>) -> Self {
        let end = start;
        Self { start, end }
    }
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
    pub fn new() -> Rc<Self> {
        let data = RefCell::new(None);
        let panning = Rc::new(Panning { data });

        let document = document().expect("document");
        let target : EventTarget = dyn_into(document).unwrap();

        use crate::system::web::console_log;

        let panning_clone = panning.clone();
        let closure = move |event:MouseEvent| {
            if event.button() == MMB {
                let mut panning = panning_clone.data.borrow_mut();
                let start = Vector2::new(event.x() as f32, event.y() as f32);
                *panning = Some(PanningData::start(start));
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
                panning.end = Vector2::new(event.x() as f32, event.y() as f32);
            }
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("mousemove", callback);
        closure.forget();

        let panning_clone = panning.clone();
        let closure = move |_:MouseEvent| {
            let mut panning = panning_clone.data.borrow_mut();
            *panning = None;
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("mouseup", callback);
        closure.forget();

        panning
    }

    pub fn consume(&self) -> Option<Vector2<f32>> {
        let mut panning = self.data.borrow_mut();
        let     panning = panning.as_mut();
        if let Some(panning) = panning {
            let res = panning.end - panning.start;
            panning.start = panning.end;
            Some(Vector2::new(-res.x, res.y))
        } else {
            None
        }
    }
}