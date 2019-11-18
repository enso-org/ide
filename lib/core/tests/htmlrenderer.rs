//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

pub mod common;

#[cfg(test)]
mod tests {
    use crate::common::TestContainer;
    use basegl::display::scene::*;
    use js_sys::Math::random;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn usage() {
        use std::f32::consts::PI;

        let _container = TestContainer::new("usage", 320.0, 240.0);
        let mut scene = HTMLScene::new("usage").expect("HTMLScene");
        assert_eq!(scene.len(), 0);

        let (width, height) = scene.get_dimension();
        assert_eq!((width, height), (320.0, 240.0));

        const TAU: f32 = PI * 2.0;

        for _ in 0 .. 51 {
            let mut object = HTMLObject::new("div").unwrap();
            object.set_position(
                ((random() - 0.5) * 200.0) as f32,
                ((random() - 0.5) * 200.0) as f32,
                ((random() - 0.5) * 200.0) as f32,
            );
            object.set_rotation(
                random() as f32 * TAU,
                random() as f32 * TAU,
                random() as f32 * TAU,
            );
            object.set_dimension(50.0, 50.0);
            object
                .element
                .style()
                .set_property(
                    "background-color",
                    &format!(
                        "rgba({}, {}, {}, {})",
                        (random() * 255.0) as u8,
                        (random() * 255.0),
                        (random() * 255.0),
                        1.0
                    ),
                )
                .expect("set background-color");
            scene.add(object);
        }
        assert_eq!(scene.len(), 51);
        scene.remove(25);
        assert_eq!(scene.len(), 50);

        let mut camera = Camera::perspective(45.0, width / height, 1.0, 2000.0);
        camera.set_position(0.0, 0.0, 200.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }

    #[wasm_bindgen_test]
    fn rhs_coordinates() {
        // Expected result: https://jsfiddle.net/zx6k7jt4/4/
        let _container = TestContainer::new("rhs_coordinates", 320.0, 240.0);
        let mut scene = HTMLScene::new("rhs_coordinates").expect("HTMLScene");
        assert_eq!(scene.len(), 0);

        scene.container.style().set_property("background-color", "black").expect("black bg");

        let (width, height) = scene.get_dimension();
        assert_eq!((width, height), (320.0, 240.0));

        for mask in vec![(1, 0, 0), (0, 1, 0), (0, 0, 1)] {
            for i in 0 .. 10 {
                let (x, y, z) = ((i * mask.0) as f32, (i * mask.1) as f32, (i * mask.2) as f32);
                let mut object = HTMLObject::new("div").unwrap();
                object.set_position(x as f32, y as f32, z as f32);
                object.set_rotation(1.0, 1.0, 1.0);
                object.set_dimension(1.0, 1.0);
                let (r, g, b) = (x as f32 * 25.5, y as f32 * 25.5, z as f32 * 25.5);
                object
                    .element
                    .style()
                    .set_property(
                        "background-color",
                        &format!("rgba({}, {}, {}, {})", r as u8, g as u8, b as u8, 1.0),
                    )
                    .expect("set background-color");
                scene.add(object);
            }
        }

        let mut camera = Camera::perspective(45.0, width / height, 1.0, 2000.0);
        camera.set_position(0.0, 0.0, 29.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }

    #[wasm_bindgen_test]
    fn rhs_coordinates_from_back() {
        use std::f32::consts::PI;

        let _container = TestContainer::new("rhs_coordinates_from_back", 320.0, 240.0);
        let mut scene = HTMLScene::new("rhs_coordinates_from_back").expect("HTMLScene");
        assert_eq!(scene.len(), 0);

        scene.container.style().set_property("background-color", "black").expect("black bg");

        let (width, height) = scene.get_dimension();
        assert_eq!((width, height), (320.0, 240.0));

        for mask in vec![(1, 0, 0), (0, 1, 0), (0, 0, 1)] {
            for i in 0 .. 10 {
                let (x, y, z) = ((i * mask.0) as f32, (i * mask.1) as f32, (i * mask.2) as f32);
                let mut object = HTMLObject::new("div").unwrap();
                object.set_position(x as f32, y as f32, z as f32);
                object.set_rotation(0.5, 0.5, 0.5);
                object.set_dimension(1.0, 1.0);
                let (r, g, b) = (x as f32 * 25.5, y as f32 * 25.5, z as f32 * 25.5);
                object
                    .element
                    .style()
                    .set_property(
                        "background-color",
                        &format!("rgba({}, {}, {}, {})", r as u8, g as u8, b as u8, 1.0),
                    )
                    .expect("set background-color");
                scene.add(object);
            }
        }

        let mut camera = Camera::perspective(45.0, width / height, 1.0, 2000.0);
        camera.set_position(0.0, 0.0, -29.0);
        camera.set_rotation(0.0, PI, 0.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }

    #[wasm_bindgen_test]
    fn object_behind_camera() {
        let _container = TestContainer::new("object_behind_camera", 320.0, 240.0);
        let mut scene = HTMLScene::new("object_behind_camera").expect("HTMLScene");
        assert_eq!(scene.len(), 0);

        let (width, height) = scene.get_dimension();
        assert_eq!((width, height), (320.0, 240.0));

        let mut object = HTMLObject::new("div").unwrap();
        object.set_position(0.0, 0.0, 0.0);
        object.element.style().set_property("background-color", "rgb(0, 0, 0)").expect("black bg");
        object.set_dimension(100.0, 100.0);
        scene.add(object);

        let mut camera = Camera::perspective(45.0, width / height, 1.0, 2000.0);
        camera.set_position(0.0, 0.0, -100.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }
}
