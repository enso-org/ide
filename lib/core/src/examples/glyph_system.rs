use crate::prelude::*;

use basegl_system_web::{forward_panic_hook_to_console, set_stdout};
use crate::display::world::*;
use crate::display::object::DisplayObjectOps;
use crate::display::shape::glyph::system::GlyphSystem;
use nalgebra::Vector4;

use wasm_bindgen::prelude::*;
use basegl_core_msdf_sys::run_once_initialized;
use crate::display::shape::glyph::font::Fonts;

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_glyph_system() {
    forward_panic_hook_to_console();
    set_stdout();
    run_once_initialized(|| init(&WorldData::new("canvas")));
}

fn init(world: &World) {
    let mut fonts        = Fonts::new();
    let font_id          = fonts.load_embedded_font("DejaVuSansMono").unwrap();
    let mut glyph_system = GlyphSystem::new(font_id);
    let line_position    = Vector2::new(0.0, 0.0);
    let height           = 30.0;
    let color            = Vector4::new(0.0, 0.8, 0.0, 1.0);
    let mut line         = glyph_system.new_line(line_position,height,"ABC",color,&mut fonts);

    world.add_child(glyph_system.sprite_system());
    world.on_frame(move |_| {
        let &_ = &line;
        glyph_system.sprite_system().update()
    }).forget();
}