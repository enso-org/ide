#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![feature(trait_alias)]
#![feature(set_stdio)]

pub mod closure;
pub mod resize_observer;
pub mod platform;

/// Common types that should be visible across the whole crate.
pub mod prelude {
    pub use enso_prelude::*;
    pub use wasm_bindgen::prelude::*;
}

use crate::prelude::*;

pub use web_sys::console;
use js_sys::Function;
use logger::Logger;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

pub use web_sys::CanvasRenderingContext2d;
pub use web_sys::Document;
pub use web_sys::Element;
pub use web_sys::EventTarget;
pub use web_sys::HtmlCanvasElement;
pub use web_sys::HtmlDivElement;
pub use web_sys::HtmlElement;
pub use web_sys::MouseEvent;
pub use web_sys::Node;
pub use web_sys::Performance;
pub use web_sys::WebGl2RenderingContext;
pub use web_sys::Window;
pub use std::time::Duration;
pub use std::time::Instant;



// =============
// === Error ===
// =============

/// Generic error representation. We may want to support errors in form of structs and enums, but it
/// requires significant work, so a simpler solution was chosen for now.
#[derive(Debug)]
pub struct Error{
    message : String
}

#[allow(non_snake_case)]
pub fn Error<S:Into<String>>(message:S) -> Error {
    let message = message.into();
    Error {message}
}

pub type Result<T> = std::result::Result<T,Error>;



// ==============
// === String ===
// ==============

#[wasm_bindgen]
extern "C" {
    /// Converts given `JsValue` into a `String`. Uses JS's `String` function,
    /// see: https://www.w3schools.com/jsref/jsref_string.asp
    #[allow(unsafe_code)]
    #[wasm_bindgen(js_name="String")]
    pub fn js_to_string(s: JsValue) -> String;
}



// =============
// === Utils ===
// =============

/// Handle returned from `ignore_context_menu`. It unignores when the handle is dropped.
#[derive(Debug)]
pub struct IgnoreContextMenuHandle {
    target  : EventTarget,
    closure : Closure<dyn FnMut(MouseEvent)>
}

impl Drop for IgnoreContextMenuHandle {
    fn drop(&mut self) {
        let callback : &Function = self.closure.as_ref().unchecked_ref();
        self.target.remove_event_listener_with_callback("contextmenu", callback).ok();
    }
}

/// Ignores context menu when clicking with the right mouse button.
pub fn ignore_context_menu(target:&EventTarget) -> Option<IgnoreContextMenuHandle> {
    let closure = move |event:MouseEvent| {
        const RIGHT_MOUSE_BUTTON : i16 = 2;
        if  event.button() == RIGHT_MOUSE_BUTTON {
            event.prevent_default();
        }
    };
    let closure = Closure::wrap(Box::new(closure) as Box<dyn FnMut(MouseEvent)>);
    let callback : &Function = closure.as_ref().unchecked_ref();
    match target.add_event_listener_with_callback("contextmenu", callback) {
        Ok(_)  => {
            let target = target.clone();
            let handle = IgnoreContextMenuHandle { target, closure };
            Some(handle)
        },
        Err(_) => None
    }
}



// ===================
// === DOM Helpers ===
// ===================

/// Access the `window` object if exists.
pub fn try_window() -> Result<Window> {
    web_sys::window().ok_or_else(|| Error("Cannot access 'window'."))
}

/// Access the `window` object or panic if it does not exist.
pub fn window() -> Window {
    try_window().unwrap()
}

/// Access the `window.document` object if exists.
pub fn try_document() -> Result<Document> {
    try_window().and_then(|w| w.document().ok_or_else(|| Error("Cannot access 'window.document'.")))
}

/// Access the `window.document` object or panic if it does not exist.
pub fn document() -> Document {
    try_document().unwrap()
}

/// Access the `window.document.body` object if exists.
pub fn try_body() -> Result<HtmlElement> {
    try_document().and_then(|d| d.body().ok_or_else(||
        Error("Cannot access 'window.document.body'.")))
}

/// Access the `window.document.body` object or panic if it does not exist.
pub fn body() -> HtmlElement {
    try_body().unwrap()
}

/// Access the `window.devicePixelRatio` value if the window exists.
pub fn try_device_pixel_ratio() -> Result<f64> {
    try_window().map(|window| window.device_pixel_ratio())
}

/// Access the `window.devicePixelRatio` or panic if the window does not exist.
pub fn device_pixel_ratio() -> f64 {
    window().device_pixel_ratio()
}

/// Access the `window.performance` or panics if it does not exist.
pub fn performance() -> Performance {
    window().performance().unwrap_or_else(|| panic!("Cannot access window.performance."))
}

pub fn get_element_by_id(id:&str) -> Result<Element> {
    try_document()?.get_element_by_id(id).ok_or_else(||
        Error(format!("Element with id '{}' not found.",id)))
}

pub fn get_html_element_by_id(id:&str) -> Result<HtmlElement> {
    let elem = get_element_by_id(id)?;
    elem.dyn_into().map_err(|_| Error("Type cast error."))
}

pub fn try_create_element(name:&str) -> Result<Element> {
    try_document()?.create_element(name).map_err(|_|
        Error(format!("Cannot create element '{}'",name)))
}

pub fn create_element(name:&str) -> Element {
    try_create_element(name).unwrap()
}

pub fn try_create_div() -> Result<HtmlDivElement> {
    try_create_element("div").map(|t| t.unchecked_into())
}

pub fn create_div() -> HtmlDivElement {
    create_element("div").unchecked_into()
}

pub fn try_create_canvas() -> Result<HtmlCanvasElement> {
    try_create_element("canvas").map(|t| t.unchecked_into())
}

pub fn create_canvas() -> HtmlCanvasElement {
    create_element("canvas").unchecked_into()
}

pub fn get_webgl2_context(canvas:&HtmlCanvasElement) -> WebGl2RenderingContext {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"antialias".into(), &false.into()).unwrap();
    let context = canvas.get_context_with_context_options("webgl2",&options).unwrap().unwrap();
    let context : WebGl2RenderingContext =  context.dyn_into().unwrap();
    context
}

pub fn try_request_animation_frame(f:&Closure<dyn FnMut(f64)>) -> Result<i32> {
    try_window()?.request_animation_frame(f.as_ref().unchecked_ref())
        .map_err(|_| Error("Cannot access 'requestAnimationFrame'."))
}

pub fn request_animation_frame(f:&Closure<dyn FnMut(f64)>) -> i32 {
    window().request_animation_frame(f.as_ref().unchecked_ref()).unwrap()
}

pub fn cancel_animation_frame(id:i32) {
    window().cancel_animation_frame(id).unwrap();
}



// =====================
// === Other Helpers ===
// =====================

/// Trait used to set HtmlElement attributes.
pub trait AttributeSetter {
    fn set_attribute_or_panic<T:Str,U:Str>(&self, name:T, value:U);

    fn set_attribute_or_warn<T:Str,U:Str>(&self, name:T, value:U, logger:&Logger);
}

impl AttributeSetter for web_sys::Element {
    fn set_attribute_or_panic<T:Str,U:Str>(&self, name:T, value:U) {
        let name   = name.as_ref();
        let value  = value.as_ref();
        let values = format!("\"{}\" = \"{}\" on \"{:?}\"",name,value,self);
        self.set_attribute(name,value)
            .unwrap_or_else(|_| panic!("Failed to set attribute {}", values));
    }

    fn set_attribute_or_warn<T:Str,U:Str>(&self, name:T, value:U, logger:&Logger) {
        let name            = name.as_ref();
        let value           = value.as_ref();
        let values          = format!("\"{}\" = \"{}\" on \"{:?}\"",name,value,self);
        let warn_msg : &str = &format!("Failed to set attribute {}", values);
        if self.set_attribute(name,value).is_err() {
            logger.warning(warn_msg)
        }
    }
}

/// Trait used to set css styles.
pub trait StyleSetter {
    fn set_style_or_panic<T:Str,U:Str>(&self, name:T, value:U);
    fn set_style_or_warn<T:Str,U:Str>(&self, name:T, value:U, logger:&Logger);
}

impl StyleSetter for web_sys::HtmlElement {
    fn set_style_or_panic<T:Str,U:Str>(&self, name:T, value:U) {
        let name      = name.as_ref();
        let value     = value.as_ref();
        let values    = format!("\"{}\" = \"{}\" on \"{:?}\"",name,value,self);
        let panic_msg = |_| panic!("Failed to set style {}",values);
        self.style().set_property(name, value).unwrap_or_else(panic_msg);
    }

    fn set_style_or_warn<T:Str,U:Str>(&self, name:T, value:U, logger:&Logger) {
        let name            = name.as_ref();
        let value           = value.as_ref();
        let values          = format!("\"{}\" = \"{}\" on \"{:?}\"",name,value,self);
        let warn_msg : &str = &format!("Failed to set style {}",values);
        if self.style().set_property(name, value).is_err() {
            logger.warning(warn_msg);
        }
    }
}

/// Trait used to insert `Node`s.
pub trait NodeInserter {
    fn append_or_panic(&self, node:&Node);

    fn append_or_warn(&self, node:&Node, logger:&Logger);

    fn prepend_or_panic(&self, node:&Node);

    fn prepend_or_warn(&self, node:&Node, logger:&Logger);

    fn insert_before_or_panic(&self,node:&Node,reference_node:&Node);

    fn insert_before_or_warn(&self,node:&Node,reference_node:&Node, logger:&Logger);
}

impl NodeInserter for Node {
    fn append_or_panic(&self, node:&Node) {
        let panic_msg = |_|
            panic!("Failed to append child {:?} to {:?}",node,self);
        self.append_child(node).unwrap_or_else(panic_msg);
    }

    fn append_or_warn(&self, node:&Node, logger:&Logger) {
        let warn_msg : &str = &format!("Failed to append child {:?} to {:?}",node,self);
        if self.append_child(node).is_err() {
            logger.warning(warn_msg)
        };
    }

    fn prepend_or_panic(&self, node:&Node) {
        let panic_msg = |_| panic!("Failed to prepend child \"{:?}\" to \"{:?}\"",node,self);
        let first_c = self.first_child();
        self.insert_before(node, first_c.as_ref()).unwrap_or_else(panic_msg);
    }

    fn prepend_or_warn(&self, node:&Node, logger:&Logger) {
        let warn_msg : &str = &format!("Failed to prepend child \"{:?}\" to \"{:?}\"",node,self);
        let first_c = self.first_child();
        if self.insert_before(node, first_c.as_ref()).is_err() {
            logger.warning(warn_msg)
        }
    }

    fn insert_before_or_panic(&self, node:&Node, ref_node:&Node) {
        let panic_msg = |_| panic!("Failed to insert {:?} before {:?} in {:?}",node,ref_node,self);
        self.insert_before(node, Some(ref_node)).unwrap_or_else(panic_msg);
    }

    fn insert_before_or_warn(&self, node:&Node, ref_node:&Node, logger:&Logger) {
        let warn_msg : &str =
            &format!("Failed to insert {:?} before {:?} in {:?}",node,ref_node,self);
        if self.insert_before(node, Some(ref_node)).is_err() {
            logger.warning(warn_msg)
        }
    }
}

/// Trait used to remove `Node`s.
pub trait NodeRemover {
    fn remove_from_parent_or_panic(&self);

    fn remove_from_parent_or_warn(&self, logger:&Logger);

    fn remove_child_or_panic(&self, node:&Node);

    fn remove_child_or_warn(&self, node:&Node, logger:&Logger);
}

impl NodeRemover for Node {
    fn remove_from_parent_or_panic(&self) {
        if let Some(parent) = self.parent_node() {
            let panic_msg = |_|  panic!("Failed to remove {:?} from parent", self);
            parent.remove_child(self).unwrap_or_else(panic_msg);
        }
    }

    fn remove_from_parent_or_warn(&self, logger:&Logger) {
        if let Some(parent) = self.parent_node() {
            let warn_msg : &str = &format!("Failed to remove {:?} from parent", self);
            if parent.remove_child(self).is_err() {
                logger.warning(warn_msg)
            }
        }
    }

    fn remove_child_or_panic(&self, node:&Node) {
        let panic_msg = |_| panic!("Failed to remove child {:?} from {:?}",node,self);
        self.remove_child(node).unwrap_or_else(panic_msg);
    }

    fn remove_child_or_warn(&self, node:&Node, logger:&Logger) {
        let warn_msg : &str = &format!("Failed to remove child {:?} from {:?}",node,self);
        if self.remove_child(node).is_err() {
            logger.warning(warn_msg)
        }
    }
}

#[wasm_bindgen(inline_js = "export function request_animation_frame2(f) { requestAnimationFrame(f) }")]
extern "C" {
    #[allow(unsafe_code)]
    pub fn request_animation_frame2(closure: &Closure<dyn FnMut()>) -> i32;
}



// ===============
// === Printer ===
// ===============

type PrintFn = fn(&str) -> std::io::Result<()>;

struct Printer {
    printfn: PrintFn,
    buffer: String,
    is_buffered: bool,
}

impl Printer {
    fn new(printfn: PrintFn, is_buffered: bool) -> Printer {
        Printer {
            buffer: String::new(),
            printfn,
            is_buffered,
        }
    }
}

impl std::io::Write for Printer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.push_str(&String::from_utf8_lossy(buf));

        if !self.is_buffered {
            (self.printfn)(&self.buffer)?;
            self.buffer.clear();

            return Ok(buf.len());
        }

        if let Some(i) = self.buffer.rfind('\n') {
            let buffered = {
                let (first, last) = self.buffer.split_at(i);
                (self.printfn)(first)?;

                String::from(&last[1..])
            };

            self.buffer.clear();
            self.buffer.push_str(&buffered);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        (self.printfn)(&self.buffer)?;
        self.buffer.clear();

        Ok(())
    }
}

fn _print(msg: &str) -> std::io::Result<()> {
    web_sys::console::info_1(&msg.to_string().into());
    Ok(())
}


pub fn set_stdout() {
    let printer = Printer::new(_print, true);
    std::io::set_print(Some(Box::new(printer)));
}

pub fn set_stdout_unbuffered() {
    let printer = Printer::new(_print, false);
    std::io::set_print(Some(Box::new(printer)));
}

#[wasm_bindgen(inline_js = "
export function set_stack_trace_limit() {
    Error.stackTraceLimit = 100
}
")]
extern "C" {
    #[allow(unsafe_code)]
    pub fn set_stack_trace_limit();
}


/// Enables forwarding panic messages to `console.error`.
pub fn forward_panic_hook_to_console() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    console_error_panic_hook::set_once();
}

/// Common traits.
pub mod traits {
    pub use super::NodeInserter;
    pub use super::NodeRemover;
}

/// Sleeps for the specified amount of time.
///
/// This function might sleep for slightly longer than the specified duration but never less.
///
/// This function is an async version of std::thread::sleep, its timer starts just after the
/// function call.
#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration:Duration) {
    use wasm_bindgen_futures::JsFuture;

    let performance       = performance();
    let call_milliseconds = performance.now();
    let future : JsFuture = js_sys::Promise::new(&mut |resolve:Function,_| {
        let milliseconds_from_call = ((performance.now() - call_milliseconds) * 1000.0) as i32;
        let duration               = duration.as_millis() as i32;
        let duration               = (duration - milliseconds_from_call).max(0);
        let window                 = window();
        let err                    = "Calling setTimeout failed.";
        window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve,duration).expect(err);
    }).into();
    // We don't expect any error coming from this Promise.
    future.await.expect("setTimeout's future failed.");
}

#[cfg(not(target_arch = "wasm32"))]
pub use async_std::task::sleep;



// ============
// === Test ===
// ============

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    #[cfg(target_arch = "wasm32")]
    mod helpers {
        type Instant = f64;

        pub fn now() -> Instant {
            super::performance().now()
        }

        pub fn elapsed(instant: Instant) -> f64 {
            super::performance().now() - instant
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    mod helpers {
        use std::time::Instant;

        pub fn now() -> Instant {
            Instant::now()
        }

        pub fn elapsed(instant: Instant) -> f64 {
            instant.elapsed().as_secs_f64()
        }
    }

    #[wasm_bindgen_test(async)]
    async fn async_sleep() {
        let instant = helpers::now();
        sleep(Duration::new(1,0)).await;
        assert!(helpers::elapsed(instant) >= 1.0);
        sleep(Duration::new(2,0)).await;
        assert!(helpers::elapsed(instant) >= 3.0);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn async_sleep_native() {
        async_std::task::block_on(async_sleep())
    }
}
