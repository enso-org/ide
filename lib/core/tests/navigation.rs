//! Test suite for the Web and headless browsers.
#![feature(trait_alias)]
#![cfg(target_arch = "wasm32")]

use web_test::web_configure;
web_configure!(run_in_browser);

#[cfg(test)]
mod tests {
    use basegl::display::camera::Camera2d;
    use basegl::system::web::dom::html::HtmlScene;
    use basegl::system::web::dom::html::HtmlObject;
    use basegl::system::web::dom::html::HtmlRenderer;
    use basegl::system::web::StyleSetter;
    use basegl::display::navigation::navigator::Navigator;
    use web_test::*;

    use nalgebra::Vector3;
    use logger::Logger;

    fn create_scene(logger:Logger) -> HtmlScene {
        let mut scene = HtmlScene::new(logger.clone());
        assert_eq!(scene.len(), 0);

        let width  = 320.0 / 2.0;
        let height = 240.0 / 2.0;

        let positions = vec![
            (0.5, 0.5),
            (0.5, 1.5),
            (1.5, 0.5),
            (1.5, 1.5)
        ];

        let colors = vec![
            (255, 0  , 0  ),
            (0  , 255, 0  ),
            (0  ,   0, 255),
            (255, 255,   0)
        ];

        for i in 0..=3 {
            let     object = HtmlObject::new(logger.clone(), "div");
            let mut object = object.expect("Couldn't create div");
            let (x, y)     = positions[i];
            object.set_dimensions(width, height);
            object.set_position(Vector3::new(width * x, height * y, 0.0));
            let (r, g, b) = colors[i];
            let color = format!("rgb({}, {}, {})", r, g, b);
            object.dom.set_property_or_panic("background-color", color);
            scene.add_child(object);
        }

        scene
    }

    fn navigator_test(b: &mut Bencher, name:&str) {
        let renderer = HtmlRenderer::new(name)
            .expect("Renderer couldn't be created");
        renderer.container().dom.set_property_or_panic("background-color", "black");

        let logger      = Logger::new("navigator_test");
        let scene = create_scene(logger.clone());

        let view_dim = renderer.dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let mut camera  = Camera2d::new(logger,view_dim.x,view_dim.y);
        camera.update();

        let fovy_slope = camera.half_fovy_slope();
        let dimensions = renderer.dimensions();
        let x = dimensions.x / 2.0;
        let y = dimensions.y / 2.0;
        let z = y / fovy_slope;
        camera.set_position(Vector3::new(x, y, z));

        let mut event_loop = b.event_loop();
        let camera_clone   = camera.clone();
        let navigator      = Navigator::new(&mut event_loop, name, camera_clone);
        let navigator      = navigator.expect("Couldn't create navigator");

        b.iter(move || {
            let _keep_alive = &navigator;
            renderer.render(&mut camera, &scene);
        })
    }

    // We create two tests to verify that each HtmlElement has its own Navigator.
    #[web_bench]
    fn navigator_test_1(b: &mut Bencher) { navigator_test(b, "navigator_test_1") }

    #[web_bench]
    fn navigator_test_2(b: &mut Bencher) { navigator_test(b, "navigator_test_2") }
}
