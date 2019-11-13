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
pub mod dirty;
pub mod display;
pub use basegl_prelude as prelude;
pub mod backend {
    pub use basegl_backend_webgl as webgl;
}
pub mod system {
    pub use basegl_system_web as web;
}

// ============
// === Main ===
// ============

use wasm_bindgen::prelude::*;
use display::world::World;
use display::scene::{HTMLScene, HTMLObject};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    let scene = HTMLScene::new("app");
    match scene {
        Ok(mut scene) => {
            for i in (0..100) {
                let mut object = HTMLObject::new("div").unwrap();
                object.set_position(js_sys::Math::random() as f32 * 600.0 - 300.0, js_sys::Math::random() as f32 * 600.0 - 300.0, js_sys::Math::random() as f32 * 800.0 - 600.0);
                object.set_rotation(js_sys::Math::random() as f32, js_sys::Math::random() as f32, js_sys::Math::random() as f32);
                object.set_inner_html("I'm an editable text!");
                object.set_attribute("contenteditable", "");
                {
                    let mut style = object.style();
                    style.set_property("background-color", &format!("rgba({}, {}, {}, {})", (js_sys::Math::random() * 255.0) as u8, (js_sys::Math::random() * 255.0), (js_sys::Math::random() * 255.0), 1.0));
                    style.set_property("width", "100px");
                    style.set_property("height", "100px");
                }
                scene.add(object);
            }
            scene.update();
        },
        Err(_) => panic!()
    }
}
