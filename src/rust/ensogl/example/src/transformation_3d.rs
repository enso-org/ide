//! Example scene showing 3D transformations on WebGL and DOM objects.

use ensogl_core::prelude::*;

use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::system::web;
use web::StyleSetter;
use wasm_bindgen::prelude::*;
use ensogl_core::display::object::ObjectOps;
use ensogl_core::display::shape::ShapeSystem;
use ensogl_core::display::world::*;
use ensogl_core::display::shape::*;
use ensogl_core::data::color;
use ensogl_core::display::DomSymbol;
use enso_frp as frp;
use ensogl_core::display;
use ensogl_text as text;
use ensogl_core::application::Application;


// ===================
// === Entry Point ===
// ===================

/// The example entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_transformation_3d() {
    web::forward_panic_hook_to_console();

    let app   = Application::new(&web::get_html_element_by_id("root").unwrap());
    let world = &app.display;
    let scene          = world.scene();
    let camera         = scene.camera().clone_ref();
    let navigator      = Navigator::new(&scene,&camera);
    let dom_back_layer = &scene.dom.layers.back;


    // === Grid ===

    let grid_div = web::create_div();
    grid_div.set_style_or_panic("width",  "1001px");
    grid_div.set_style_or_panic("height", "1001px");
    grid_div.set_style_or_panic("background-size", "25px 25px");
    grid_div.set_style_or_panic("background-image",
        "linear-gradient(to right,  grey 1px, transparent 1px),
         linear-gradient(to bottom, grey 1px, transparent 1px)");

    let grid = DomSymbol::new(&grid_div);
    dom_back_layer.manage(&grid);
    world.add_child(&grid);
    grid.set_size(Vector2(1001.0,1001.0));


    // === WebGL Square ===

    let webgl_square = display::object::Instance::new(Logger::new(""));
    world.add_child(&webgl_square);
    webgl_square.mod_position(|t| *t = Vector3::new(0.0, 75.0, 0.0));

    let shape         = AnyShape::from(Rect((&100.px(),&100.px())).fill(color::Rgba::blue()));
    let sprite_system = ShapeSystem::new(scene,&shape);
    world.add_child(&sprite_system);
    let sprite = sprite_system.new_instance();
    webgl_square.add_child(&sprite);
    sprite.size.set(Vector2::new(110.0, 110.0));

    let label = text::Area::new(&app);
    webgl_square.add_child(&label);
    label.set_content("EnsoGL object");
    label.set_position_xy(Vector2(-50.0,50.0));


    // === DOM Square ===

    let div = web::create_div();
    div.set_inner_html("DOM node");
    div.set_style_or_panic("width", "100px");
    div.set_style_or_panic("height", "100px");
    div.set_style_or_panic("background-color", "#ff0000");

    let dom_square = DomSymbol::new(&div);
    dom_back_layer.manage(&dom_square);
    world.add_child(&dom_square);
    dom_square.set_size(Vector2(100.0, 100.0));
    dom_square.mod_position(|t| *t = Vector3::new(0.0, -75.0, 0.0));


    // === Camera Rotation ===

    let network = frp::Network::new("");
    frp::extend! { network
        mouse_dragged <- scene.mouse.frp.translation.gate(&scene.mouse.frp.is_down_primary);
        camera_angle  <- any_mut::<Vector2>();
        camera_angle  <+ mouse_dragged.map2(&camera_angle,
            |translation,angle| Vector2(angle.x-translation.y/1000.0,angle.y+translation.x/1000.0));
        eval camera_angle((angle) camera.set_rotation_xy(*angle));
    }


    // === On Frame ===

    world.on_frame(move |time| {
        let _keep_alive = &navigator;
        let _keep_alive = &grid;
        let _keep_alive = &webgl_square;
        let _keep_alive = &sprite;
        let _keep_alive = &label;
        let _keep_alive = &dom_square;
        let _keep_alive = &network;

        webgl_square.set_rotation(Vector3(0.0, time.local/1000.0, 0.0));
        dom_square.set_rotation(Vector3(0.0, time.local/1000.0, 0.0));
    }).forget();
}
