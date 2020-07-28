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

// TODO [MM]: Connect to doc parser


#[wasm_bindgen]
#[allow(dead_code)]
#[allow(clippy::many_single_char_names)]
pub fn run_example_documentation_generator_view() {
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
    div.set_inner_html("<h1>hello baseGL</h1>");

    let width  = screen.width;
    let height = screen.height;

    let mut css3d_objects: Vec<DomSymbol> = default();
    let size       = Vector2::new(width, height);
    let position   = Vector3::new(0.0, 0.0, 0.0);
    let object     = DomSymbol::new(&div);
    dom_front_layer.manage(&object);
    world.add_child(&object);
    let r          = (32.0) as u8;
    let g          = (32.0) as u8;
    let b          = (32.0) as u8;
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
