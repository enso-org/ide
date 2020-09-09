//! A debug scene showing the bug described in https://github.com/enso-org/ide/issues/757

use crate::prelude::*;

use ensogl_core::system::web;
use ensogl_core::application::Application;
use ensogl_core::gui::component::Animation;
use ensogl_text_msdf_sys::run_once_initialized;
use logger::enabled::Logger;
use wasm_bindgen::prelude::*;



// ===================
// === Entry Point ===
// ===================

/// An entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_animation() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    run_once_initialized(|| {
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());
        init();
        mem::forget(app);
    });
}


// ========================
// === Init Application ===
// ========================

fn init() {

    let logger  = Logger::new("AnimationTest");
    let network = enso_frp::Network::new();
    let animation = Animation::<f32>::new(&network);
    animation.set_target_value(-259_830.0);

    enso_frp::extend! {network
        eval animation.value([logger](value) {
            info!(logger, "Value {value}")
        });
    }
    std::mem::forget(animation);
    std::mem::forget(network);
}
