use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent;
use std::fmt::Debug;
use failure::_core::fmt::{Formatter, Error};

mod js {
    use wasm_bindgen::prelude::*;
    use web_sys::KeyboardEvent;

    #[wasm_bindgen(module = "/src/system/web/text_input/text_input.js")]
    extern "C" {
        pub type TextInputHandlers;

        #[allow(unsafe_code)]
        #[wasm_bindgen(constructor)]
        pub fn new() -> TextInputHandlers;

        #[allow(unsafe_code)]
        #[wasm_bindgen(method)]
        pub fn set_copy_handler(this:&TextInputHandlers, handler:&Closure<dyn FnMut(bool) -> String>);

        #[allow(unsafe_code)]
        #[wasm_bindgen(method)]
        pub fn set_paste_handler(this:&TextInputHandlers, handler:&Closure<dyn FnMut(String)>);

        #[allow(unsafe_code)]
        #[wasm_bindgen(method)]
        pub fn set_event_handler
        (this:&TextInputHandlers, name:&str, handler:&Closure<dyn FnMut(KeyboardEvent)>);

        #[allow(unsafe_code)]
        #[wasm_bindgen(method)]
        pub fn stop_handling(this:&TextInputHandlers);
    }
}

pub trait CopyHandler          = FnMut(bool) -> String + 'static;
pub trait PasteHandler         = FnMut(String) + 'static;
pub trait KeyboardEventHandler = FnMut(KeyboardEvent) + 'static;

pub struct KeyboardBinding {
    js_handlers      : js::TextInputHandlers,
    copy_handler     : Option<Closure<dyn CopyHandler>>,
    paste_handler    : Option<Closure<dyn PasteHandler>>,
    key_down_handler : Option<Closure<dyn KeyboardEventHandler>>,
    key_up_handler   : Option<Closure<dyn KeyboardEventHandler>>,
}

impl KeyboardBinding {
    pub fn new() -> Self {
        KeyboardBinding {
            js_handlers      : js::TextInputHandlers::new(),
            copy_handler     : None,
            paste_handler    : None,
            key_down_handler : None,
            key_up_handler   : None
        }
    }

    pub fn set_copy_handler<Handler:CopyHandler>(&mut self, handler:Handler) {
        let handler_js : Closure<dyn CopyHandler> = Closure::wrap(Box::new(handler));
        self.js_handlers.set_copy_handler(&handler_js);
        self.copy_handler = Some(handler_js);
    }

    pub fn set_paste_handler<Handler:PasteHandler>(&mut self, handler:Handler) {
        let handler_js : Closure<dyn PasteHandler> = Closure::wrap(Box::new(handler));
        self.js_handlers.set_paste_handler(&handler_js);
        self.paste_handler = Some(handler_js);
    }

    pub fn set_key_down_handler<Handler:KeyboardEventHandler>(&mut self, handler:Handler) {
        let handler_js : Closure<dyn KeyboardEventHandler> = Closure::wrap(Box::new(handler));
        self.js_handlers.set_event_handler("keydown", &handler_js);
        self.key_down_handler = Some(handler_js);
    }

    pub fn set_key_up_handler<Handler:KeyboardEventHandler>(&mut self, handler:Handler) {
        let handler_js : Closure<dyn KeyboardEventHandler> = Closure::wrap(Box::new(handler));
        self.js_handlers.set_event_handler("keyup", &handler_js);
        self.key_up_handler = Some(handler_js);
    }
}

impl Drop for KeyboardBinding {
    fn drop(&mut self) {
        self.js_handlers.stop_handling();
    }
}

impl Debug for KeyboardBinding {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("<KeyboardBindings>")
    }
}
