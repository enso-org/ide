use wasm_bindgen::prelude::*;

use crate::display::world::{WorldData, Workspace, Add};
use crate::display::shape::text::font::FontId;
use crate::display::shape::text::Color;
use crate::data::dirty::traits::*;

use itertools::iproduct;
use nalgebra::{Point2,Vector2};

const FONT_NAMES : &[&str] = &
    [ "DejaVuSans"
        , "DejaVuSansMono"
        , "DejaVuSansMono-Bold"
        , "DejaVuSerif"
    ];

const SIZES : &[f64] = &[0.024, 0.032, 0.048];

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_text() {
    set_panic_hook();
    basegl_core_msdf_sys::run_once_initialized(|| {
        let mut world_ref = WorldData::new("canvas");
        let world :&mut WorldData = &mut world_ref.borrow_mut();
        let workspace     = &mut world.workspace;
        let fonts         = &mut world.fonts;
        let font_ids_iter = FONT_NAMES.iter().map(|name| fonts.load_embedded_font(name).unwrap());
        let font_ids      = font_ids_iter.collect::<Box<[FontId]>>();

        let all_cases     = iproduct!(0..font_ids.len(), 0..SIZES.len());

        for (font, size) in all_cases {

            let x = -0.95 + 0.6 * (size as f64);
            let y = 0.90 - 0.45 * (font as f64);
            let text_compnent = crate::display::shape::text::TextComponentBuilder {
                workspace,
                fonts,
                text : "To be, or not to be, that is the question:\n\
                    Whether 'tis nobler in the mind to suffer\n\
                    The slings and arrows of outrageous fortune,\n\
                    Or to take arms against a sea of troubles\n\
                    And by opposing end them."
                    .to_string(),
                font_id: font_ids[font],
                position: Point2::new(x, y),
                size: Vector2::new(0.5, 0.2),
                text_size: SIZES[size],
                color    : Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0},
            }.build();
            workspace.text_components.push(text_compnent);
        }
        world.workspace_dirty.set();

//            world.on_frame(move |w| {
//                let space = &mut w.workspace;
//                for text_component in &mut space.text_components {
//                    text_component.scroll(Vector2::new(0.0,0.00001));
//                }
//                w.workspace_dirty.set();
//            }).forget();
    });
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
}
