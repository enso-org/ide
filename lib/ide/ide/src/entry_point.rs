#![allow(missing_docs)]

use wasm_bindgen::prelude::*;
use basegl::system::web;

use super::project_view::ProjectView;

#[wasm_bindgen]
#[allow(dead_code)]
pub fn ide_startup() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    basegl_core_msdf_sys::run_once_initialized(|| {
        entry_point()
    });
}

fn entry_point() {
    ProjectView::new().forget();
}