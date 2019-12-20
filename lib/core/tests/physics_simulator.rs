//! Test suite for the Web and headless browsers.
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
    use basegl::animation::physics::{DragProperties, SpringProperties, PhysicsObject};
    use basegl::animation::Animator;
    use basegl::animation::physics::{PhysicsSimulator, PhysicsProperties};
    use basegl::prelude::default;
    use basegl::traits::HasPosition;
    use web_test::*;
    use nalgebra::{zero, Vector3};
    use js_sys::Math::random;

    #[web_bench]
    fn simulator(b : &mut Bencher) {
        let renderer = HTMLRenderer::new("simulator").expect("Renderer couldn't be created");
        renderer.container.dom.set_property_or_panic("background-color", "black");

        let mut scene : Scene<HTMLObject> = Scene::new();

        let mut target = HTMLObject::new("div").unwrap();
        target.set_dimensions(1.0, 1.0);
        target.dom.set_property_or_panic("background-color", "green");
        scene.add(target.clone());

        let mut object = HTMLObject::new("div").unwrap();
        object.set_dimensions(1.0, 1.0);
        object.dom.set_property_or_panic("background-color", "red");
        scene.add(object.clone());

        let view_dim = renderer.dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
        camera.set_position(Vector3::new(0.0, 0.0, 29.0));

        let kinematics     = default();
        let coefficient    = 10.0;
        let fixed_point    = zero();
        let spring         = SpringProperties::new(coefficient, fixed_point);
        let drag           = DragProperties::new(0.01);
        let mut properties = PhysicsProperties::new(kinematics, spring, drag);
        let mass           = 1.0;
        let physics_object = PhysicsObject::new(object, mass);
        let simulator      = PhysicsSimulator::new(physics_object, properties.clone());

        // Updates spring's fixed point every two seconds.
        let every = 2.0;
        let animator  = Animator::new(1.0 / every, move |_| {
            let x = 32.0 * (random() - 0.5) as f32;
            let y = 24.0 * (random() - 0.5) as f32;
            let z = 0.0;
            let position = Vector3::new(x, y, z);
            properties.mod_spring(|spring| spring.set_fixed_point(position));
            target.set_position(position);
        });

        std::mem::forget(simulator);
        std::mem::forget(animator);
        b.iter(move || {
            renderer.render(&mut camera, &scene);
        });
    }
}
