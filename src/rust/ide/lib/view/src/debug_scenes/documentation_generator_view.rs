#![allow(missing_docs)]

use ensogl::traits::*;
use ensogl::prelude::*;

use ensogl::system::web;
use ensogl::system::web::NodeInserter;
use ensogl::display::symbol::DomSymbol;
use web::StyleSetter;
use ensogl::display::world::*;
use ensogl::display::navigation::navigator::Navigator;

use nalgebra::Vector2;
use nalgebra::Vector3;
use wasm_bindgen::prelude::*;

use crate::graph_editor::component::visualization::MockDocGenerator;
use crate::graph_editor::builtin::visualization::native::documentation::doc_style;

fn generate_mock_doc() -> String {
    let sample_data_gen = MockDocGenerator::default();
    let default_input   = sample_data_gen.generate_data();
    let program         = std::env::args().nth(1).unwrap_or(default_input);

    let parser = parser::DocParser::new_or_panic();
    let output = parser.generate_html_docs(program);
    let output = output.unwrap_or_else(|_| String::from("<h1>hello EnsoGL</h1>"));
    let output = output.replace(r#"<link rel="stylesheet" href="style.css" />"#, "");
    format!(r#"<div class="docVis">{}{}</div>"#, doc_style(), output)
}


#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_documentation_generator_view() {
    let mock_doc = generate_mock_doc();

    web::forward_panic_hook_to_console();
    web::set_stdout();
    let world           = World::new(&web::get_html_element_by_id("root").unwrap());
    let scene           = world.scene();
    let camera          = scene.camera();
    let screen          = camera.screen();
    let navigator       = Navigator::new(scene,camera);
    let dom_front_layer = &scene.dom.layers.main;
    let dom_back_layer  = &scene.dom.layers.overlay;

    let div = web::create_div();
    div.set_style_or_panic("width" ,"100% !important");
    div.set_style_or_panic("height","100% !important");
    div.set_inner_html(&mock_doc);

    let width  = screen.width;
    let height = screen.height;

    let mut css3d_objects: Vec<DomSymbol> = default();
    let size       = Vector2::new(width, height);
    let position   = Vector3::new(0.0, 0.0, 0.0);
    let object     = DomSymbol::new(&div);
    dom_front_layer.manage(&object);
    world.add_child(&object);
    let color = "rgb(255.0,255.0,255.0)";
    div.set_style_or_panic("background-color",color);

    object.dom().append_or_panic(&div);
    object.set_size(size);
    object.mod_position(|t| *t = position);
    css3d_objects.push(object);

    let layers = vec![dom_front_layer.clone_ref(),dom_back_layer.clone_ref()];

    world.keep_alive_forever();
    world.on_frame(move |_| {
        let _keep_alive = &navigator;

        for (_, object) in css3d_objects.iter_mut().enumerate() {
            layers[0].manage(&object);
        }
    }).forget();
}
