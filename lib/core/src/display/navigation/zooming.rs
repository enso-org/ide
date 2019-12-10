// FIXME: NEEDS MAJOR REFACTORING!

use crate::system::web::document;
use crate::system::web::dyn_into;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Function;
use web_sys::MouseEvent;
use web_sys::EventTarget;
use std::rc::Rc;
use std::cell::RefCell;
use nalgebra::Vector2;

// ============
// === Zoom ===
// ============

pub struct Zoom {
    pub panning : Vector2<f32>,
    pub amount  : f32
}

impl Zoom {
    pub fn new(panning:Vector2<f32>, amount:f32) -> Self {
        Self { panning, amount }
    }
}

// ===================
// === ZoomingData ===
// ===================

struct ZoomingData {
    focus : Vector2<f32>,
    start : f32,
    end   : f32
}

impl ZoomingData {
    fn start(focus:Vector2<f32>, start:f32) -> Self {
        let end = start;
        Self { focus, start, end }
    }
}

// ===============
// === Zooming ===
// ===============

pub struct Zooming {
    data : RefCell<Option<ZoomingData>>
}

const LMB : i16 = 0;
const MMB : i16 = 1; // MMB for middle mouse button? :P
const RMB : i16 = 2;

impl Zooming {
    pub fn new() -> Rc<Self> {
        let data = RefCell::new(None);
        let zooming = Rc::new(Zooming { data });

        let document = document().expect("document");
        let target : EventTarget = dyn_into(document).unwrap();

        let zooming_clone = zooming.clone();
        let closure = move |event:MouseEvent| -> bool {
            if event.button() == LMB {
                event.prevent_default();
                let mut zooming = zooming_clone.data.borrow_mut();
                let focus = Vector2::new(event.x() as f32, event.y() as f32);
                let start = focus.y;
                *zooming = Some(ZoomingData::start(focus, start));
                false
            } else {
                false
            }
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent) -> bool>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("mousedown", callback);
        closure.forget();

        let zooming_clone = zooming.clone();
        let closure = move |event:MouseEvent| {
            let mut zooming = zooming_clone.data.borrow_mut();
            let     zooming = zooming.as_mut();
            if let Some(zooming) = zooming {
                zooming.end = event.y() as f32;
            }
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("mousemove", callback);
        closure.forget();

        let zooming_clone = zooming.clone();
        let closure = move |_:MouseEvent| {
            let mut zooming = zooming_clone.data.borrow_mut();
            *zooming = None;
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("mouseup", callback);
        closure.forget();

        zooming
    }

    pub fn consume(&self, factor : f32) -> Option<Zoom> {
        let mut zooming = self.data.borrow_mut();
        let     zooming = zooming.as_mut();
        if let Some(zooming) = zooming {
            let mut amount   = zooming.end - zooming.start;
            zooming.start = zooming.end;

            let amount = if amount < 0.0 {
                -1.0 / (amount * factor - 1.0)
            } else if amount > 0.0 {
                amount * factor + 1.0
            } else {
                1.0
            };

            // hardcoded values
            let offset = Vector2::new(21.0, 185.0);
            let dimension = Vector2::new(320.0, 240.0);
            let center = dimension / 2.0;

            let point       = zooming.focus - offset;
            let delta       = point - center;
            let new_delta   = delta * amount;
            let mut panning = delta - new_delta;
            panning.y       = -panning.y;
            let zoom        = Zoom::new(panning, amount);
            Some(zoom)
        } else {
            None
        }
    }
}