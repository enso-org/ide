//! Example scene showing simple usage of a shape system.

use crate::prelude::*;
use wasm_bindgen::prelude::*;

use ensogl::display::navigation::navigator::Navigator;
use ensogl::system::web;
use ensogl::display::object::ObjectOps;
use ensogl::application::Application;

use ensogl_text_msdf_sys::run_once_initialized;

// ===================
// === Entry Point ===
// ===================

/// The example entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_close_button() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    run_once_initialized(|| {
        println!("Initializing");
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());

        let shape:crate::top_buttons::close::View = app.new_view();
        shape.model.shape.size.set(Vector2::new(300.0, 300.0));
        app.display.add_child(&shape);

        let scene         = app.display.scene();
        let camera        = scene.camera().clone_ref();
        let navigator     = Navigator::new(&scene,&camera);

        let network = enso_frp::Network::new("test");
        // let logger : Logger = Logger::new("CloseButton");
        // enso_frp::extend! {network
        //     eval shape.clicked([logger](()) {
        //         info!(logger, "Clicked")
        //     });
        //     eval shape.faux_clicked([logger](()) {
        //         info!(logger, "Faux Clicked")
        //     });
        // }

        std::mem::forget(shape);
        std::mem::forget(network);
        std::mem::forget(navigator);
        mem::forget(app);
        println!("Initialized");
    });
}
