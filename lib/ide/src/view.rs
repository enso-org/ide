//! A module containing view components.

#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

pub mod temporary_panel;
pub mod project;
pub mod layout;
pub mod text_editor;

use crate::prelude::*;

use basegl::system::web::document;
use js_sys::Function;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use web_sys::KeyboardEvent;



// ========================
// === KeyboardListener ===
// ========================
// This code is temporary and will be replace with FRP keyboard events once it's fully functional.

type KeyboardClosure = Closure<dyn FnMut(KeyboardEvent)>;

#[derive(Debug)]
struct KeyboardListener {
    callback   : KeyboardClosure,
    element    : HtmlElement,
    event_type : String
}

impl KeyboardListener {
    fn new(event_type:String, callback:KeyboardClosure) -> Self {
        let element                 = document().unwrap().body().unwrap();
        let js_function : &Function = callback.as_ref().unchecked_ref();
        element.add_event_listener_with_callback(&event_type,js_function).ok();
        Self {callback,element,event_type}
    }
}

impl Drop for KeyboardListener {
    fn drop(&mut self) {
        let callback : &Function = self.callback.as_ref().unchecked_ref();
        self.element.remove_event_listener_with_callback(&self.event_type, callback).ok();
    }
}
