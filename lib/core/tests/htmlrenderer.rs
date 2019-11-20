//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

pub mod common;

#[cfg(test)]
mod tests {
    use crate::common::TestContainer;
    use basegl::display::rendering::*;
    use basegl::system::web::StyleSetter;
    use js_sys::Math::random;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn usage() {
        use std::f32::consts::PI;

        let _container = TestContainer::new("usage", 320.0, 240.0);
        let mut scene = HTMLScene::new("usage")
                                  .expect("Failed to create HTMLScene");
        assert_eq!(scene.len(), 0);

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        const TAU: f32 = PI * 2.0;

        for _ in 0 .. 51 {
            let mut object = HTMLObject::new("div").unwrap();

            let x = ((random() - 0.5) * 200.0) as f32;
            let y = ((random() - 0.5) * 200.0) as f32;
            let z = ((random() - 0.5) * 200.0) as f32;
            object.set_position(x, y, z);

            let roll  = random() as f32 * TAU;
            let pitch = random() as f32 * TAU;
            let yaw   = random() as f32 * TAU;
            object.set_rotation(roll, pitch, yaw);
            object.set_dimensions(50.0, 50.0);

            let r = (random() * 255.0) as u8;
            let g = (random() * 255.0) as u8;
            let b = (random() * 255.0) as u8;
            let color = format!("rgb({}, {}, {}, 1.0)", r, g, b);
            object.element.set_property_or_panic("background-color", color);
            scene.add(object);
        }
        assert_eq!(scene.len(), 51);
        scene.remove(25);
        assert_eq!(scene.len(), 50);

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
        camera.set_position(0.0, 0.0, 200.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }

    #[wasm_bindgen_test]
    fn rhs_coordinates() {
        // Note [rhs expected result]
        // https://jsfiddle.net/zx6k7jt4/4/
        TestContainer::new("rhs_coordinates", 320.0, 240.0);
        let mut scene = HTMLScene::new("rhs_coordinates")
                                  .expect("Failed to create HTMLScene");
        assert_eq!(scene.len(), 0);

        scene
            .container
            .set_property_or_panic("background-color", "black");

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        for mask in vec![(1, 0, 0), (0, 1, 0), (0, 0, 1)] {
            for i in 0 .. 10 {
                let mut object = HTMLObject::new("div").unwrap();

                let x = (i * mask.0) as f32;
                let y = (i * mask.1) as f32;
                let z = (i * mask.2) as f32;
                object.set_position(x + 10.0, y + 10.0, z);
                object.set_rotation(0.5, 0.5, 0.5);
                object.set_dimensions(1.0, 1.0);

                let r = (x * 25.5) as u8;
                let g = (y * 25.5) as u8;
                let b = (z * 25.5) as u8;
                let color = format!("rgba({}, {}, {}, {})", r, g, b, 1.0);

                object.element.set_property_or_panic("background-color", color);
                scene.add(object);
            }
        }

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
        camera.set_position(10.0, 10.0, 29.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }

    #[wasm_bindgen_test]
    fn rhs_coordinates_from_back() {
        use std::f32::consts::PI;

        TestContainer::new("rhs_coordinates_from_back", 320.0, 240.0);
        let mut scene = HTMLScene::new("rhs_coordinates_from_back")
                                  .expect("Failed to create HTMLScene");
        assert_eq!(scene.len(), 0);

        scene
            .container
            .set_property_or_panic("background-color", "black");

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        for mask in vec![(1, 0, 0), (0, 1, 0), (0, 0, 1)] {
            for i in 0 .. 10 {
                let mut object = HTMLObject::new("div").unwrap();

                let x = (i * mask.0) as f32;
                let y = (i * mask.1) as f32;
                let z = (i * mask.2) as f32;
                object.set_position(x + 10.0, y + 10.0, z);
                object.set_rotation(0.5, 0.5, 0.5);
                object.set_dimensions(1.0, 1.0);

                let r = (x * 25.5) as u8;
                let g = (y * 25.5) as u8;
                let b = (z * 25.5) as u8;
                let color = format!("rgba({}, {}, {}, {})", r, g, b, 1.0);

                object.element.set_property_or_panic("background-color", color);
                scene.add(object);
            }
        }

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
        camera.set_position(10.0, 10.0, -29.0);
        camera.set_rotation(0.0, PI, 0.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }

    #[wasm_bindgen_test]
    fn object_behind_camera() {
        TestContainer::new("object_behind_camera", 320.0, 240.0);
        let mut scene = HTMLScene::new("object_behind_camera")
                                  .expect("Failed to create HTMLScene");
        assert_eq!(scene.len(), 0);

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let mut object = HTMLObject::new("div").unwrap();
        object.set_position(0.0, 0.0, 0.0);
        object.element.set_property_or_panic("background-color", "black");
        object.set_dimensions(100.0, 100.0);
        scene.add(object);

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
        camera.set_position(0.0, 0.0, -100.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }
}
