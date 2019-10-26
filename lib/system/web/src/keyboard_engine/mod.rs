pub mod callback_registry;
pub mod bindings;

use std::cell::Cell;
use bigint::uint::U256;
use web_sys::{KeyboardEvent};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::*;
use self::bindings::*;
use self::callback_registry::*;

/// Keyboard Engine that handles user keyboard combinations
///
/// Keyboard Engine stores bindings as HashMap:
/// {
///     U256:          [Callback1, Callback2],
///     U256:          [Callback1],
/// }
/// when the combination is pressed related callbacks are invoked
/// Pressed combination is encoded into U256 bitstring
pub struct KeyboardEngine {
    pub bindings: Rc<Bindings>,
    pub state:    Rc<Cell<U256>>,
    pub target:   web_sys::EventTarget
}

impl KeyboardEngine {
    pub fn new(target: &web_sys::EventTarget) -> Self {
        let mut this = Self {
            bindings: Rc::new(Bindings::new()),
            state:    Rc::new(Cell::new(U256::from(0))),
            target:   target.clone()
        };
        this.init();
        this
    }

    pub fn from_tag_name(html_tag: &str) -> Result<Self> {
        let element = document()?
            .get_elements_by_tag_name(html_tag)
            .item(0).ok_or_else(|| Error::missing("there is no such element"));

        Ok(KeyboardEngine::new(&element.unwrap()))
    }

    fn init(&mut self) {
        self.bind_keydown_event();
        self.bind_keyup_event();
    }

    fn bind_keydown_event(&self) {
        let state = Rc::clone(&self.state);
        let bindings = Rc::clone(&self.bindings);
        let callback = Box::new(move |event: web_sys::KeyboardEvent| {
            if event.repeat() {
                return;
            }

            let key_code = event.key_code();
            let bit = U256::from(1) << (key_code as usize);
            let key = state.get() | bit;
            state.set(key);
            bindings.call_by_key(key);
        });
        let callback = Closure::wrap(callback as Box<dyn FnMut(_)>);

        let callback_ref = callback.as_ref().unchecked_ref();
        self.target
            .add_event_listener_with_callback("keydown", callback_ref).unwrap();
        callback.forget();
    }

    fn bind_keyup_event(&self) {
        let bindings = Rc::clone(&self.bindings);
        let state =  Rc::clone(&self.state);
        let callback = Box::new(move |event: KeyboardEvent| {
            let key_code = event.key_code();
            let bit = U256::from(1) << (key_code as usize);
            let key = state.get() ^ bit;
            state.set(key);
            bindings.call_by_key(key);
        });
        let callback = Closure::wrap(callback as Box<dyn FnMut(_)>);

        let callback_ref = callback.as_ref().unchecked_ref();
        self.target
            .add_event_listener_with_callback("keyup", callback_ref).unwrap();
        callback.forget();
    }

    /// Returns U256 number that represents bitstring of pressed keys
    fn get_key_bits(&self, combination: Vec<u32>) -> U256 {
        combination.iter()
            .fold(U256::from(0), |acc, x| {
                let bit = U256::from(1) << (*x as usize);
                acc | bit
            })
    }

    /// Capture key combination
    ///
    /// Uses u32 as key code from WebIDL
    /// https://github.com/rustwasm/wasm-bindgen/blob/913fdbc3daff65952b5678a34b98e07d4e6e4fbb/crates/web-sys/webidls/enabled/KeyEvent.webidl
    pub fn capture<F: CallbackMut>
    (&self, combination: Vec<u32>, callback: F) -> CallbackHandle {
        let key = self.get_key_bits(combination);
        self.bindings.add(key, callback)
    }

    /// Drops watching specific combination
    pub fn drop_capture(&self, combination: Vec<u32>) {
        let key = self.get_key_bits(combination);
        self.bindings.remove(key);
    }
}
