//! An example showing usage of GlyphSystem.

use basegl::display::object::DisplayObject;
use basegl::display::object::DisplayObjectOps;
use basegl::display::shape::text::glyph::font::FontRegistry;
use basegl::display::shape::text::glyph::system::GlyphSystem;
use basegl::display::world::*;
use basegl::system::web;
use basegl_core_msdf_sys::run_once_initialized;
use nalgebra::Vector4;
use wasm_bindgen::prelude::*;



/// Main example runner.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_glyph_system() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    run_once_initialized(|| init(&WorldData::new(&web::body())));
}

fn init(world: &World) {
    let mut fonts        = FontRegistry::new();
    let font             = fonts.get_or_load_embedded_font("DejaVuSans").unwrap();
    let mut glyph_system = GlyphSystem::new(world,font);
    let line_position    = Vector2::new(100.0, 100.0);
    let height           = 32.0;
    let color            = Vector4::new(0.0, 0.8, 0.0, 1.0);
    let text             = "Follow  the white rabbit...";
    let line             = glyph_system.new_line(line_position,height,text,color);

    world.add_child(glyph_system.sprite_system());
    world.on_frame(move |_| {
        let &_ = &line;
        glyph_system.sprite_system().display_object().update();
    }).forget();
}
