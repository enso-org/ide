#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use ensogl::prelude::*;

use enso_frp as frp;
use ensogl::application::Application;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::display::object::ObjectOps;
use ensogl::system::web;
use graph_editor::GraphEditor;
use graph_editor::component::visualization;
use serde_json::json;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_shapes() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    let app = Application::new(&web::get_html_element_by_id("root").unwrap());
    init(&app);
    mem::forget(app);
}

fn set_up_visualization_demo_frp(graph_editor: &GraphEditor) {
    let network       = &graph_editor.frp.network;
    let dummy_counter = Rc::new(Cell::new(1.0_f32));
    frp::extend! { network
        def _set_dumy_data = graph_editor.frp.inputs.set_visualization_data.map(move |node| {
            let dc = dummy_counter.get();
            dummy_counter.set(dc + 0.1);
            let content = json!(format!("{}", 20.0 + 10.0 * dummy_counter.get().sin()));
            let dummy_data = Some(visualization::Data::JSON { content });
            node.visualization.frp.set_data.emit(dummy_data);
        });
    };
}

fn init(app:&Application) {
    let world     = &app.display;
    let scene     = world.scene();
    let camera    = scene.camera();
    let navigator = Navigator::new(&scene,&camera);

    app.views.register::<GraphEditor>();
    let graph_editor = app.views.new::<GraphEditor>();
    world.add_child(&graph_editor);

     // Visualisation data dummy functionality
    set_up_visualization_demo_frp(&graph_editor);

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
