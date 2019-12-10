// FIXME: NEEDS MAJOR REFACTORING!

use crate::system::web::document;
use crate::system::web::dyn_into;

use crate::display::rendering::DOMContainer;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Function;
use web_sys::MouseEvent;
use web_sys::EventTarget;
use std::rc::Rc;
use std::cell::RefCell;
use nalgebra::Vector2;

use crate::system::web::console_log;

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
    dom  : DOMContainer,
    data : RefCell<Option<ZoomingData>>
}

const LMB : i16 = 0;
const MMB : i16 = 1; // MMB for middle mouse button? :P
const RMB : i16 = 2;

impl Zooming {
    pub fn new(dom:&DOMContainer) -> Rc<Self> {
        let data    = RefCell::new(None);
        let dom     = dom.clone();
        let zooming = Rc::new(Zooming { dom, data });

        let target : EventTarget = dyn_into(zooming.dom.dom.clone()).unwrap();

        let zooming_clone = zooming.clone();
        let closure = move |event:MouseEvent| {
            if event.button() == RMB {
                event.prevent_default();
            }
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("contextmenu", callback);
        closure.forget();

        let zooming_clone = zooming.clone();
        let closure = move |event:MouseEvent| {
            if event.button() == RMB {
                let mut zooming = zooming_clone.data.borrow_mut();
                let focus = Vector2::new(event.x() as f32, event.y() as f32);
                let start = focus.y;
                *zooming = Some(ZoomingData::start(focus, start));
            }
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
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

        let zooming_clone = zooming.clone();
        let closure = move |_:MouseEvent| {
            let mut zooming = zooming_clone.data.borrow_mut();
            *zooming = None;
        };
        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(MouseEvent)>);
        let callback : &Function = closure.as_ref().unchecked_ref();
        target.add_event_listener_with_callback("mouseleave", callback);
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

            let position    = self.dom.position();
            let dimension   = self.dom.dimensions();
            let center      = dimension / 2.0;
            let point       = zooming.focus - position;
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