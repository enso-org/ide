//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

pub mod common;

#[cfg(test)]
mod tests {
	use wasm_bindgen_test::*;
	use crate::common::{TestContainer};
	use js_sys::Math::random;

	#[wasm_bindgen_test]
	fn usage() {
		use basegl::display::scene::*;

		let scene = HTMLScene::new("invalid_parent");
		assert!(scene.is_err(), "HtmlElement with id=invalid_parent doesn't exist");

		let container = TestContainer::new("usage", 320.0, 240.0);
		let mut scene = HTMLScene::new("usage").expect("HTMLScene");
		assert_eq!(scene.len(), 0);

		let (width, height) = scene.get_dimension();
		assert_eq!((width, height), (320.0, 240.0));

		for _ in 0..51 {
	        let mut object = HTMLObject::new("div").unwrap();
	        object.set_position(((random() - 0.5) * 200.0) as f32, ((random() - 0.5) * 200.0) as f32, ((random() - 0.5) * 200.0) as f32);
	        object.set_rotation(random() as f32, random() as f32, random() as f32);
	        object.set_dimension(50.0, 50.0);
	        object.element.style().set_property("background-color", &format!("rgba({}, {}, {}, {})", (random() * 255.0) as u8, (random() * 255.0), (random() * 255.0), 1.0)).expect("set background-color");
	        scene.add(object);
	    }
		assert_eq!(scene.len(), 51);
		scene.remove(25);
		assert_eq!(scene.len(), 50);

		let mut camera = Camera::perspective(45.0, width / height, 1.0, 2000.0);

		let renderer = HTMLRenderer::new();
		renderer.render(&mut camera, &scene);
	}

	#[wasm_bindgen_test]
	fn other() {
		use basegl::display::scene::*;

		let scene = HTMLScene::new("invalid_parent");
		assert!(scene.is_err(), "HtmlElement with id=invalid_parent doesn't exist");

		let container = TestContainer::new("other", 320.0, 240.0);
		let mut scene = HTMLScene::new("other").expect("HTMLScene");
		assert_eq!(scene.len(), 0);

		let (width, height) = scene.get_dimension();
		assert_eq!((width, height), (320.0, 240.0));

		for _ in 0..51 {
			let mut object = HTMLObject::new("div").unwrap();
			object.set_position(((random() - 0.5) * 200.0) as f32, ((random() - 0.5) * 200.0) as f32, ((random() - 0.5) * 200.0) as f32);
			object.set_rotation(random() as f32, random() as f32, random() as f32);
			object.set_dimension(50.0, 50.0);
			object.element.style().set_property("background-color", &format!("rgba({}, {}, {}, {})", (random() * 255.0) as u8, (random() * 255.0), (random() * 255.0), 1.0)).expect("set background-color");
			scene.add(object);
		}
		assert_eq!(scene.len(), 51);
		scene.remove(25);
		assert_eq!(scene.len(), 50);

		let mut camera = Camera::perspective(45.0, width / height, 1.0, 2000.0);

		let renderer = HTMLRenderer::new();
		renderer.render(&mut camera, &scene);
	}
}
