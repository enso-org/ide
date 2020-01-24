//! This module contains implementation of a mouse manager and related utilities.

use crate::prelude::*;

use crate::system::web::dom::DOMContainer;
use crate::system::web::dyn_into;
use crate::system::web::Result;
use crate::system::web::Error;
use crate::system::web::ignore_context_menu;

use js_sys::Function;
use nalgebra::Vector2;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::Closure;
use web_sys::AddEventListenerOptions;
use web_sys::EventTarget;
use web_sys::WheelEvent;
use web_sys::HtmlElement;
use crate::control::callback::*;
use basegl_prelude::default;



// ===================
// === MouseButton ===
// ===================

/// An enumeration representing the mouse buttons. Please note that we do not name the buttons
/// left, right, and middle, as this assumes we use a mouse for right-hand people.
///
/// JS supports up to 5 mouse buttons currently:
/// https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button
#[derive(Debug,Clone,Copy)]
pub enum Button {_0,_1,_2,_3,_4}

impl Button {
    pub fn from_code(code:i16) -> Self {
        match code {
            0 => Self::_0,
            1 => Self::_1,
            2 => Self::_2,
            3 => Self::_3,
            4 => Self::_4,
            _ => panic!("Invalid button code"),
        }
    }
}



// =============
// === Event ===
// =============

#[derive(Debug,Clone,From,Shrinkwrap)]
pub struct Event {
    raw: web_sys::MouseEvent
}
impl Event {
    /// Translation of the button property to Rust `Button` enum.
    pub fn button(&self) -> Button {
        Button::from_code(self.raw.button())
    }
}



// =======================
// === EventDispatcher ===
// =======================

pub trait MouseEventFn      = Fn(web_sys::MouseEvent) + 'static;
pub type  MouseEventClosure = Closure<dyn Fn(JsValue)>;

fn mouse_event_closure<F:MouseEventFn>(f:F) -> MouseEventClosure {
    Closure::wrap(Box::new(move |event:JsValue| {
        let event = event.unchecked_into::<web_sys::MouseEvent>();
        f(event)
    }))
}

#[derive(Debug,Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Default(bound=""))]
pub struct EventDispatcher<T> {
    rc: Rc<RefCell<XCallbackRegistry1<T>>>
}

impl<T> EventDispatcher<T> {
    pub fn add<F:XCallbackMut1Fn<T>>(&self, callback:F) -> CallbackHandle {
        self.rc.borrow_mut().add(callback)
    }

    pub fn dispatch(&self, t:&T) {
        self.rc.borrow_mut().run_all(t);
    }
}

impl<T> CloneRef for EventDispatcher<T> {
    fn clone_ref(&self) -> Self {
        self.clone()
    }
}



// ====================
// === MouseManager ===
// ====================

#[derive(Debug,Shrinkwrap)]
pub struct MouseManager {
    #[shrinkwrap(main_field)]
    dispatchers : MouseManagerDispatchers,
    closures    : MouseManagerClosures,
    dom         : EventTarget,
}

macro_rules! define_bindings {
    ( $( $js_name:ident => $name:ident),* $(,)? ) => {

        #[derive(Debug,Default)]
        pub struct MouseManagerDispatchers {
            $(pub $name : EventDispatcher<Event>),*
        }

        #[derive(Debug)]
        pub struct MouseManagerClosures {
            $(pub $name : MouseEventClosure),*
        }

        impl MouseManager {
            pub fn new(dom:&EventTarget) -> Self {
                let dispatchers = MouseManagerDispatchers::default();
                let dom         = dom.clone();
                $(
                    let dispatcher = dispatchers.$name.clone_ref();
                    let $name      = mouse_event_closure(move |event:web_sys::MouseEvent| {
                        dispatcher.dispatch(&event.into())
                    });
                    let js_closure = $name.as_ref().unchecked_ref();
                    let js_name    = stringify!($js_name);
                    let result     = dom.add_event_listener_with_callback(js_name,js_closure);
                    match result {
                        Err(e) => panic!("Cannot add event listener. {:?}",e),
                        _      => {}
                    }
                )*
                let closures = MouseManagerClosures {$($name),*};
                Self {dispatchers,closures,dom}
            }
        }
    };
}

define_bindings! {
    mousedown => on_down,
    mouseup   => on_up,
    mousemove => on_move,
}
