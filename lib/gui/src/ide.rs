//! This module defines the entrypoint function for IDE.

use wasm_bindgen::prelude::*;

use basegl::system::web;
use ide::entry_point::entry_point;

/// IDE startup function.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_ide() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    basegl_core_msdf_sys::run_once_initialized(|| {
        web::get_element_by_id("loader").map(|t| {
            t.parent_node().map(|p| {
                p.remove_child(&t).unwrap()
            })
        }).ok();

        entry_point()
    });
}
