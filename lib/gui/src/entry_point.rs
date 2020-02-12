use basegl::system::web;
use ide::entry_point::entry_point;

use wasm_bindgen::prelude::*;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_ide_startup() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    basegl_core_msdf_sys::run_once_initialized(|| {
        entry_point()
    });
}