#![feature(type_ascription)]
#![feature(unboxed_closures)]
#![cfg_attr(test, allow(dead_code))]
//#![warn(missing_docs)]

// Lints. To be refactored after this gets resolved: https://github.com/rust-lang/cargo/issues/5034
#![allow(clippy::option_map_unit_fn)]

#[macro_use] extern crate shrinkwraprs;

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
use display::scene::{HTMLScene, HTMLObject, HTMLRenderer, Camera};
use system::web::{request_animation_frame, window};
use wasm_bindgen::JsCast;
use nalgebra::{Vector3};

#[wasm_bindgen(start)]
pub fn start() {
    let renderer = HTMLRenderer::new();
    let mut camera = Camera::new();

    let mut scene = HTMLScene::new("app").unwrap();
    let (width, height) = scene.get_dimension();
    for _ in 0..100 {
        let mut object = HTMLObject::new("div").unwrap();
        object.set_position(js_sys::Math::random() as f32 * width - width / 2.0, js_sys::Math::random() as f32 * height - height / 2.0, js_sys::Math::random() as f32 * 800.0);
        object.set_rotation(js_sys::Math::random() as f32, js_sys::Math::random() as f32, js_sys::Math::random() as f32);
        object.set_dimension(100.0, 100.0);
        object.element.set_inner_html("I'm an editable text!");
        object.element.set_attribute("contenteditable", "").unwrap();
        object.element.style().set_property("background-color", &format!("rgba({}, {}, {}, {})", (js_sys::Math::random() * 255.0) as u8, (js_sys::Math::random() * 255.0), (js_sys::Math::random() * 255.0), 0.5)).unwrap();
        scene.add(object);
    }
    let mut object = HTMLObject::new("div").unwrap();
    object.set_position(js_sys::Math::random() as f32 * width - width / 2.0, js_sys::Math::random() as f32 * height - height / 2.0, js_sys::Math::random() as f32 * 800.0 - 600.0);
    object.set_rotation(js_sys::Math::random() as f32, js_sys::Math::random() as f32, js_sys::Math::random() as f32);
    object.element.set_inner_html("<iframe width=\"560\" height=\"315\" src=\"https://www.youtube.com/embed/fAvPpS3Ds90\" frameborder=\"0\" allow=\"accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture\" allowfullscreen></iframe>");
    object.set_dimension(560.0, 315.0);
    scene.add(object);
    let mut t = 0.0;
    let cb = Closure::wrap(Box::new(move || {
//        camera.position = camera.position + Vector3::new(0.0, 0.0, 0.1);
        camera.set_rotation(0.0, t, 0.0);
        t += 0.001;
        let mut z = 0.0;
        for object in &mut scene.objects.items {
            z += 0.01;
            if let Some(object) = object {
                object.set_rotation(t + z, t + z, t + z);
            }
        }

        renderer.render(&mut camera, &mut scene);
    }) as Box<dyn FnMut()>);
    window().unwrap().set_interval_with_callback(cb.as_ref().unchecked_ref());
    cb.forget();
}
