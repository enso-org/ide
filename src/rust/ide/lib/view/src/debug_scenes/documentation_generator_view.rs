#![allow(missing_docs)]

use ensogl::traits::*;

use ensogl::system::web;
use ensogl::system::web::NodeInserter;
use ensogl::display::symbol::DomSymbol;
use web::StyleSetter;
use ensogl::display::world::*;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::prelude::*;

use nalgebra::Vector2;
use nalgebra::Vector3;
use wasm_bindgen::prelude::*;

fn generate_mock_doc() -> String {
    let inp_only_doc = r#"DEPRECATED
                             REMOVED - replaced by Foo Bar
                             ADDED
                             MODIFIED
                             UPCOMING
                             ALAMAKOTA a kot ma Ale
                             This is a test of Enso Documentation Parser. This is a short synopsis.

                             Here you can write the body of documentation. On top you can see tags
                             added to this piece of code. You can customise your text with _Italic_
                             ~Strikethrough~ or *Bold*. ~_*Combined*_~ is funny


                             There are 3 kinds of sections
                               - Important
                               - Info
                               - Example
                                 * You can use example to add multiline code to your documentation

                             ! Important
                               Here is a small test of Important Section

                             ? Info
                               Here is a small test of Info Section

                             > Example
                               Here is a small test of Example Section
                                   Import Foo
                                   def Bar a
                             "#;
    let default_input = String::from(inp_only_doc);
    let program = std::env::args().nth(1).unwrap_or(default_input);

    let parser = parser::DocParser::new_or_panic();
    let output = parser.generate_html_doc_pure(program);
    let output_unwrapped = output.unwrap_or(String::from("<h1>hello EnsoGL</h1>"));
    output_unwrapped
}


#[wasm_bindgen]
#[allow(dead_code)]
#[allow(clippy::many_single_char_names)]
pub fn run_example_documentation_generator_view() {
    let output_unwrapped = generate_mock_doc();

    web::forward_panic_hook_to_console();
    web::set_stdout();
    let world         = World::new(&web::get_html_element_by_id("root").unwrap());
    let scene         = world.scene();
    let camera        = scene.camera();
    let screen        = camera.screen();
    let navigator     = Navigator::new(scene,camera);
    let dom_front_layer = &scene.dom.layers.main;
    let dom_back_layer  = &scene.dom.layers.overlay;

    let div = web::create_div();
    div.set_style_or_panic("width"  , "100%");
    div.set_style_or_panic("height" , "100%");
    div.set_inner_html(&output_unwrapped);

    let width  = screen.width;
    let height = screen.height;

    let mut css3d_objects: Vec<DomSymbol> = default();
    let size       = Vector2::new(width, height);
    let position   = Vector3::new(0.0, 0.0, 0.0);
    let object     = DomSymbol::new(&div);
    dom_front_layer.manage(&object);
    world.add_child(&object);
    let r          = (250.0) as u8;
    let g          = (250.0) as u8;
    let b          = (250.0) as u8;
    let color      = iformat!("rgb({r},{g},{b})");
    div.set_style_or_panic("background-color",color);

    object.dom().append_or_panic(&div);
    object.set_size(size);
    object.mod_position(|t| *t = position);
    css3d_objects.push(object);

    world.display_object().update();

    let layers = vec![dom_front_layer.clone_ref(),dom_back_layer.clone_ref()];

    world.keep_alive_forever();
    world.on_frame(move |_| {
        let _keep_alive = &navigator;

        for (_, object) in css3d_objects.iter_mut().enumerate() {
            layers[0].manage(&object);
        }
    }).forget();
}
