//! Test suite for the Web and headless browsers.
#![cfg(target_arch = "wasm32")]

use web_test::web_configure;
web_configure!(run_in_browser);

#[cfg(test)]
mod tests {
    use basegl::system::web::StyleSetter;
    use basegl::animation::physics::inertia::DragProperties;
    use basegl::animation::physics::inertia::SpringProperties;
    use basegl::animation::physics::inertia::KinematicsProperties;
    use basegl::animation::physics::inertia::PhysicsSimulator;
    use basegl::animation::physics::inertia::PhysicsProperties;
    use basegl::animation::animator::fixed_step::FixedStepAnimator;
    use basegl::system::web::dom::html::Css3dRenderer;
    use basegl::system::web::dom::html::Css3dObject;
    use web_test::*;
    use nalgebra::{zero, Vector3};
    use js_sys::Math::random;
    use basegl::display::world::WorldData;
    use basegl::display::object::DisplayObjectOps;
    use basegl::system::web::dyn_into;
    use basegl::system::web::get_element_by_id;
    use web_sys::HtmlElement;
    use nalgebra::Vector2;
    use basegl::system::web::set_stdout;
    use basegl::display::object::DisplayObject;
    use basegl::system::web;

    #[web_test]
    fn simulator() {
        set_stdout();
        let name      = "simulator";
        let container = dyn_into::<_,HtmlElement>(get_element_by_id(name).unwrap()).unwrap();
        let world     = WorldData::new(&container);
        let scene     = world.scene();
        let renderer  = scene.css3d_renderer();

        container.set_style_or_panic("background-color", "black");

        let div = web::create_div();
        div.set_style_or_panic("width"  , "100%");
        div.set_style_or_panic("height" , "100%");
        let mut target = Css3dObject::new(&div);
        renderer.manage(&target);
        target.set_dimensions(Vector2::new(10.0, 10.0));
        div.set_style_or_panic("background-color", "green");

        let div = web::create_div();
        div.set_style_or_panic("width"  , "100%");
        div.set_style_or_panic("height" , "100%");
        let mut object = Css3dObject::new(&div);
        renderer.manage(&object);
        object.set_dimensions(Vector2::new(10.0, 10.0));
        div.set_style_or_panic("background-color", "red");

        let mass             = 2.0;
        let position         = object.position();
        let kinematics       = KinematicsProperties::new(position, zero(), zero(), mass);
        let coefficient      = 10.0;
        let fixed_point      = zero();
        let spring           = SpringProperties::new(coefficient, fixed_point);
        let drag             = DragProperties::new(0.8);
        let mut properties   = PhysicsProperties::new(kinematics, spring, drag);
        let steps_per_second = 60.0;
        let simulator        = PhysicsSimulator::new(
            steps_per_second,
            properties.clone(),
            move |position| {
                object.mod_position(|t| *t = position);
                world.display_object().update();
            }
        );

        // Updates spring's fixed point every two seconds.
        let every    = 2.0;
        let animator = FixedStepAnimator::new(1.0 / every, move |_| {
            let _keep_alive = &simulator;
            let _keep_alive = &renderer;

            let x = 320.0 * random() as f32;
            let y = 240.0 * random() as f32;
            let z = 0.0;
            let position = Vector3::new(x, y, z);
            properties.modify_spring(|spring| spring.fixed_point = position);
            target.mod_position(|t| *t = position);
        });

        std::mem::forget(animator);
    }
}
