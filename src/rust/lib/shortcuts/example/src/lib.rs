
use enso_prelude::*;
use wasm_bindgen::prelude::*;
use ensogl_system_web as web;

#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_shortcuts() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();

    println!("hello2");
}
