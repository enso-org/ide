#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use ensogl::prelude::*;

use ensogl::display::navigation::navigator::Navigator;
use ensogl::system::web;
use ensogl::application::Application;
use graph_editor::GraphEditor;
use wasm_bindgen::prelude::*;
use ensogl::display::object::ObjectOps;
use ensogl_core_msdf_sys::run_once_initialized;
use ensogl::display::style::theme;
use ensogl::data::color;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_shapes() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    run_once_initialized(|| {
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());
        init(&app);
        mem::forget(app);
    });
}

fn init(app:&Application) {

    let mut dark = theme::Theme::new();
    dark.insert("application.background.color", color::Lcha::new(0.53,104.0,0.11,1.0));
    dark.insert("animation.duration", 0.5);
    dark.insert("graph.node.shadow.color", 5.0);
    dark.insert("graph.node.shadow.size", 5.0);
    dark.insert("mouse.pointer.color", color::Rgba::new(0.3,0.3,0.3,1.0));

    app.themes.register("dark",dark);
    app.themes.set_enabled(&["dark"]);

    let bg = app.display.scene().style_sheet.var("application.background.color");

    println!("{:?}",bg.value());
    println!("{:?}",app.display.scene().style_sheet.debug_sheet_nodes_count());

//    let t1 : color::Hsla = color::Hsla::new(0.0,0.0,0.03,1.0);
//    let t2 : color::Lcha = t1.into();
//    let t4 : color::Rgba = color::Rgba::from(t2);
//    println!("{:?}", t2);
//    println!("{:?}", color::Rgba::from(t1));
//    println!("{:?}", t4);
//    println!("{:?}", color::Hsla::from(color::LinearRgba::new(0.2,0.3,0.4,1.0)));
//
//    let x = color::Hsla::from(color::Rgba::new(0.031,0.031,0.031,1.0));
//    let y = color::Rgba::from(x);
//    println!("{:?}", y);
    let xyz = color::Xyz::from(color::Rgb::new(0.2,0.4,0.6));
    let lab = color::Lab::from(color::Rgb::new(0.2,0.4,0.6));
    let lch = color::Lch::from(color::Rgb::new(0.2,0.4,0.6));
    let lch = color::Lch::from(color::Rgb::new(1.0,0.0,0.0));
    println!("{:?}", xyz);
    println!("{:?}", lab);
    println!("{:?}", lch);
    println!("-----------");
    println!("{:?}", color::Rgb::from(xyz));
    println!("{:?}", color::Rgb::from(lab));
    println!("{:?}", color::Rgb::from(lch));
//    println!("{:?}", color::Lab::from(color::Xyz::new(0.1,0.2,0.3)));

    println!("{:?}", palette::Xyz::from(palette::Srgb::new(0.2,0.4,0.6)));
//    println!("{:?}", palette::Lab::from(palette::LinSrgb::new(0.2,0.4,0.6)));
//    println!("{:?}", palette::Lab::from(palette::Xyz::new(0.1,0.2,0.3)));


//    color::test();

    let world     = &app.display;
    let scene     = world.scene();
    let camera    = scene.camera();
    let navigator = Navigator::new(&scene,&camera);

    app.views.register::<GraphEditor>();
    let graph_editor = app.views.new::<GraphEditor>();
    world.add_child(&graph_editor);

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

