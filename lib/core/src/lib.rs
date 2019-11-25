#![feature(type_ascription)]
#![feature(unboxed_closures)]
#![cfg_attr(test, allow(dead_code))]
//#![warn(missing_docs)]

// Lints. To be refactored after this gets resolved: https://github.com/rust-lang/cargo/issues/5034
#![allow(clippy::option_map_unit_fn)]

// =================================
// === Module Structure Reexport ===
// =================================

pub mod data;
pub mod math;
pub mod dirty;
pub mod display;
pub mod text;
pub use basegl_prelude as prelude;
pub mod backend {
    pub use basegl_backend_webgl as webgl;
}
pub mod system {
    pub use basegl_system_web as web;
}
pub mod utils;

// ============
// === Main ===
// ============

use display::world::World;
use wasm_bindgen::prelude::*;
use basegl_core_embedded_fonts::EmbeddedFonts;
use crate::text::font::FontRenderInfo;
use itertools::iproduct;

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
    basegl_core_msdf_sys::set_library_initialized_callback(|| {
        let world = World::new();
        let workspace_id = world.add_workspace("canvas");

        {
            let mut world_data = world.data.borrow_mut();
            let workspace = world_data.workspaces.items[workspace_id].as_mut().unwrap();

            let font_base = EmbeddedFonts::create_and_fill();
            let mut fonts = [
                FontRenderInfo::from_embedded(&font_base, "DejaVuSans".to_string()),
                FontRenderInfo::from_embedded(&font_base, "DejaVuSansMono".to_string()),
                FontRenderInfo::from_embedded(&font_base, "DejaVuSansMono-Bold".to_string()),
                FontRenderInfo::from_embedded(&font_base, "DejaVuSansMono-Oblique".to_string()),
                FontRenderInfo::from_embedded(&font_base, "DejaVuSansCondensed".to_string()),
                FontRenderInfo::from_embedded(&font_base, "DejaVuSerif".to_string()),
                FontRenderInfo::from_embedded(&font_base, "DejaVuSerifCondensed".to_string()),
            ];
            let sizes = [0.024, 0.032, 0.048, 0.064];

            for (i, (font, size)) in iproduct!(0..fonts.len(), sizes.iter()).enumerate() {
                let text_compnent = crate::text::TextComponent::new(
                    workspace.data.clone(),
                    "To be, or not to be, that is the question: Whether 'tis nobler in the mind to suffer The slings and arrows of outrageous fortune, Or to take arms against a sea of troubles And by opposing end them.".to_string(),
                    -0.95,
                    0.9 - 0.064*(i as f32),
                    *size,
                    &mut fonts[font],
                    text::Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0},
                    text::Color {r: 0.0, g: 0.0, b: 0.0, a: 1.0}
                );
                workspace.text_components.push(text_compnent);
            }
            workspace.dirty.set();
        }

        world.start();
    });
}
