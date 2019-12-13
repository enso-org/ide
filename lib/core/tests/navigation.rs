//! Test suite for the Web and headless browsers.
#![feature(trait_alias)]
#![cfg(target_arch = "wasm32")]

use web_test::web_configure;
web_configure!(run_in_browser);

#[cfg(test)]
mod tests {
    use basegl::display::rendering::Scene;
    use basegl::display::rendering::Camera;
    use basegl::display::rendering::html::HTMLObject;
    use basegl::display::rendering::html::HTMLRenderer;
    use basegl::system::web::StyleSetter;
    use basegl::system::web::get_performance;
    use web_test::*;
    use basegl::display::navigation::Navigator;

    use nalgebra::Vector3;

    fn create_scene() -> Scene<HTMLObject> {
        let mut scene : Scene<HTMLObject> = Scene::new();
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
            let     object = HTMLObject::new("div");
            let mut object = object.expect("Couldn't create div");
            let (x, y)     = positions[i];
            object.set_dimensions(width, height);
            object.set_position(width * x, height * y, 0.0);
            let (r, g, b) = colors[i];
            let color = format!("rgb({}, {}, {})", r, g, b);
            object.dom.set_property_or_panic("background-color", color);
            scene.add(object);
        }

        scene
    }

    fn navigator_test(b: &mut Bencher, name:&str) {
        let renderer = HTMLRenderer::new(name)
            .expect("Renderer couldn't be created");
        renderer.container.dom.set_property_or_panic("background-color", "black");

        let scene = create_scene();

        let view_dim = renderer.dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let mut camera  = Camera::perspective(45.0, 320.0 / 240.0, 1.0, 1000.0);
        let performance = get_performance()
                         .expect("Couldn't get performance obj");

        let dimensions = renderer.dimensions();
        let x = dimensions.x / 2.0;
        let y = dimensions.y / 2.0;
        let z = y * camera.get_y_scale();
        *camera.position_mut() = Vector3::new(x, y, z);

        let navigator     = Navigator::new(&renderer.container, &camera);
        let mut navigator = navigator.expect("Couldn't create navigator");

        let mut t0 = (performance.now() / 1000.0) as f32;
        b.iter(move || {
            let t1 = (performance.now() / 1000.0) as f32;
            let dt = t1 - t0;
            t0 = t1;
            navigator.navigate(&mut camera, dt);
            renderer.render(&mut camera, &scene);
        })
    }

    // We create two tests to verify that each HtmlElement has its own
    // Navigator.
    #[web_bench]
    fn navigator_test_1(b: &mut Bencher) { navigator_test(b, "navigator_test_1") }

    #[web_bench]
    fn navigator_test_2(b: &mut Bencher) { navigator_test(b, "navigator_test_2") }
}
