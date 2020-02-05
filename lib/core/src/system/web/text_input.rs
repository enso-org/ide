use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use web_sys::KeyboardEvent;

mod internal {
    use wasm_bindgen::prelude::*;
    use web_sys::KeyboardEvent;
    use web_sys::HtmlElement;

    #[wasm_bindgen(module = "/src/system/web/text_input/text_input.js")]
    extern "C" {
        #[allow(unsafe_code)]
        pub fn bind_keyboard_events
        ( copy_handler     : &Closure<dyn FnMut() -> String>
        , paste_handler    : &Closure<dyn FnMut(String)>
        , key_down_handler : &Closure<dyn FnMut(KeyboardEvent)>
        , key_up_handler   : &Closure<dyn FnMut(KeyboardEvent)>
        ) -> HtmlElement;
    }

}

#[derive(Debug)]
pub struct KeyboardBinding {
    dom : HtmlElement,
    copy_handler  : Closure<dyn FnMut() -> String>,
    paste_handler : Closure<dyn FnMut(String)>,
    event_handler : Closure<dyn FnMut(KeyboardEvent)>,
}

impl KeyboardBinding {
    pub fn new<CopyHandler,PasteHandler,EventHandler>
    (copy_handler:CopyHandler, paste_handler:PasteHandler, event_handler:EventHandler)
    -> Self
    where CopyHandler  : FnMut() -> String + 'static,
          PasteHandler : FnMut(String) + 'static,
          EventHandler : FnMut(KeyboardEvent) + 'static {
        println!("Binding create");
        let copy_handler_js  = Closure::<dyn FnMut() -> String>::wrap(Box::new(copy_handler));
        let paste_handler_js = Closure::<dyn FnMut(String)>::wrap(Box::new(paste_handler));
        let event_handler_js = Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(event_handler));
        KeyboardBinding {
            dom : internal::bind_keyboard_events(&copy_handler_js,&paste_handler_js,&event_handler_js,&event_handler_js),
            copy_handler  : copy_handler_js,
            paste_handler : paste_handler_js,
            event_handler : event_handler_js,
        }
    }
}

impl Drop for KeyboardBinding {
    fn drop(&mut self) {
        println!("Binding drop");
    }
}
