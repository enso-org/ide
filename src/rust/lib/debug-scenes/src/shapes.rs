#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use ensogl::prelude::*;

use ensogl::display::navigation::navigator::Navigator;
use ensogl::display::world::*;
use ensogl::system::web;
use ensogl::app::App;
use graph_editor::GraphEditor;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use ensogl::display::object::ObjectOps;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_shapes() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    let app = App::new(&web::get_html_element_by_id("root").unwrap());
    init(&app);
    mem::forget(app);
}


fn init(app:&App) {
    let world     = &app.display;
    let scene     = world.scene();
    let camera    = scene.camera();
    let navigator = Navigator::new(&scene,&camera);

    app.views.register::<GraphEditor>();
    let graph_editor = app.views.new::<GraphEditor>();
    world.add_child(&graph_editor);

    let add_node_ref = graph_editor.frp.add_node_at_cursor.clone_ref();
    let remove_selected_nodes_ref = graph_editor.frp.remove_selected_nodes.clone_ref();
    let selected_nodes2 = graph_editor.nodes.selected.clone_ref();

    let mut was_rendered = false;
    let mut loader_hidden = false;
    world.on_frame(move |_| {
        let _keep_alive = &navigator;
        let _keep_alive = &graph_editor;

        // Temporary code removing the web-loader instance.
        // To be changed in the future.
        if was_rendered && !loader_hidden {
            web::get_element_by_id("loader").map(|t| {
                t.parent_node().map(|p| {
                    p.remove_child(&t).unwrap()
                })
            }).ok();
            loader_hidden = true;
        }
        was_rendered = true;
    }).forget();
}
