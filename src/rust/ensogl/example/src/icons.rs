//! Example scene showing simple usage of a shape system.

use ensogl_core::prelude::*;

use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::system::web;
use wasm_bindgen::prelude::*;
use ensogl_core::display::object::ObjectOps;
use ensogl_core::display::world::*;
use ensogl_core::data::color;
use ensogl_core::display::style::theme;
use ensogl_core::display::DomSymbol;
use ensogl_web::StyleSetter;
use ensogl_core::display::camera::Camera2d;
use ensogl_gui_components::file_browser;
use ensogl_gui_components::file_browser::icons::DynamicIcon;


// ===================
// === Entry Point ===
// ===================

/// The example entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_icons() {
    web::forward_panic_hook_to_console();
    web::set_stack_trace_limit();

    let world     = World::new(&web::get_html_element_by_id("root").unwrap());
    let scene     = world.scene();
    let camera: Camera2d = scene.camera().clone_ref();
    let navigator = Navigator::new(&scene,&camera);

    let theme_manager = theme::Manager::from(&scene.style_sheet);

    let theme1 = theme::Theme::new();
    theme1.set("base_color", color::Rgba::new(0.0,0.0,1.0,1.0));
    theme1.set("animation.duration", 0.5);
    theme1.set("graph.node.shadow.color", 5.0);
    theme1.set("graph.node.shadow.size", 5.0);
    theme1.set("mouse.pointer.color", color::Rgba::new(0.3,0.3,0.3,1.0));

    let theme2 = theme::Theme::new();
    theme2.set("base_color", color::Rgba::new(0.0,1.0,0.0,1.0));
    theme2.set("animation.duration", 0.7);
    theme2.set("graph.node.shadow.color", 5.0);
    theme2.set("graph.node.shadow.size", 5.0);
    theme2.set("mouse.pointer.color", color::Rgba::new(0.3,0.3,0.3,1.0));

    theme_manager.register("theme1",theme1);
    theme_manager.register("theme2",theme2);

    theme_manager.set_enabled(&["theme1".to_string()]);


    // === Grid ===

    let grid_div = web::create_div();
    grid_div.set_style_or_panic("width",  "1001px");
    grid_div.set_style_or_panic("height", "1001px");
    grid_div.set_style_or_panic("background-size", "0.5px 0.5px");
    grid_div.set_style_or_panic("background-image",
                                "linear-gradient(to right,  grey 0.02px, transparent 0.02px),
                                 linear-gradient(to bottom, grey 0.02px, transparent 0.02px)");

    let grid = DomSymbol::new(&grid_div);
    &scene.dom.layers.back.manage(&grid);
    world.add_child(&grid);
    grid.set_size(Vector2(50.0,50.0));
    grid.set_position_y(50.0);
    mem::forget(grid);



    let style_watch = ensogl_core::display::shape::StyleWatch::new(&scene.style_sheet);
    style_watch.set_on_style_change(|| DEBUG!("Style changed!"));
    style_watch.get("base_color");

    let icon = file_browser::icons::Arrow::new();
    icon.set_color(color::Rgba::black());
    world.add_child(&icon);
    mem::forget(icon);

    world.keep_alive_forever();
    mem::forget(navigator);
    mem::forget(style_watch);
    mem::forget(theme_manager);
}
