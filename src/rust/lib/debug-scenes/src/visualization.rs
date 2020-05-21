#![allow(missing_docs)]

use ensogl::display::navigation::navigator::Navigator;
use ensogl::system::web;
use ensogl::application::Application;
use wasm_bindgen::prelude::*;
use ensogl_core_msdf_sys::run_once_initialized;
use graph_editor::component::visualization::NativeConstructorClass;
use graph_editor::component::visualization::ClassAttributes;
use graph_editor::component::visualization::Visualization;
use graph_editor::component::visualization::JsRenderer;
use graph_editor::component::visualization::Data;
use graph_editor::component::visualization::Registry;
use std::rc::Rc;
use ensogl::display::Scene;
use nalgebra::Vector2;
use js_sys::Math::sin;
// use ide::controller::visualization::Handle;

fn generate_data(seconds:f64) -> Vec<Vector2<f32>> {
    let mut data = Vec::new();
    for x in 0..100 {
        let x = x as f64 / 50.0 - 1.0;
        let y = sin(x * std::f64::consts::PI + seconds);
        data.push(Vector2::new(x as f32,y as f32));
    }
    data
}

fn constructor_sample_js_bubble_chart() -> JsRenderer {
    let fn_constructor = r#"
        class Graph {
            onDataReceived(root, data) {
                if (!root.canvas) {
                    root.canvas  = document.createElement("canvas");
                    root.context = root.canvas.getContext("2d");
                    root.appendChild(root.canvas);
                }

                let first = data.shift();
                if (first) {
                    root.context.clearRect(0,0,root.canvas.width,root.canvas.height);
                    root.context.save();
                    root.context.scale(root.canvas.width/2,root.canvas.height/2);
                    root.context.translate(1,1);
                    root.context.lineWidth = 1/Math.min(root.canvas.width,root.canvas.height);
                    root.context.beginPath();
                    root.context.moveTo(first[0],first[1]);
                    data.forEach(data => {
                        root.context.lineTo(data[0],data[1]);
                    });
                    root.context.stroke();
                    root.context.restore();
                    root.context.beginPath();
                    root.context.moveTo(first[0],first[1]);
                    root.context.stroke();
                }
            }

            setSize(root, size) {
                if (root.canvas) {
                    root.canvas.width  = size[0];
                    root.canvas.height = size[1];
                }
            }
        }

        return new Graph();
    "#;
    JsRenderer::from_constructor(fn_constructor).unwrap()
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_visualization() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    run_once_initialized(|| {
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());
        init(&app);
        std::mem::forget(app);
    });
}

fn init(app:&Application) {
    let world      = &app.display;
    let scene      = world.scene();
    let camera     = scene.camera();
    let navigator  = Navigator::new(&scene,&camera);
    let registry   = Registry::with_default_visualizations();

    registry.register_class(NativeConstructorClass::new(
        ClassAttributes {
            name        : "Graph (JS)".to_string(),
            input_types : vec!["[[float;2]]".to_string().into()],
        },
        |scene:&Scene| {
            let renderer = constructor_sample_js_bubble_chart();
            renderer.set_dom_layer(&scene.dom.layers.front);
            Ok(Visualization::new(renderer))
        }
    ));

    let vis_factories = registry.valid_sources(&"[[float;2]]".into());
    let vis_class     = vis_factories.iter().find(|class| {
        class.attributes().name == "Graph (JS)"
    }).expect("Couldn't find Graph (JS) class.");
    let visualization = vis_class.instantiate(&scene).expect("Couldn't create visualiser.");

    let mut was_rendered = false;
    let mut loader_hidden = false;
    world.on_frame(move |time_info| {
        let _keep_alive = &navigator;

        let data    = generate_data((time_info.local / 1000.0).into());
        let data    = Rc::new(data);
        let content = Rc::new(serde_json::to_value(data).unwrap());
        let data    = Data::JSON{ content };

        visualization.frp.set_data.emit(Some(data));

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
