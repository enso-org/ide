use wasm_bindgen::prelude::*;

use crate::display::render::css3d::Scene;
use crate::display::render::css3d::Camera;
use crate::display::render::css3d::html::HTMLObject;
use crate::display::render::css3d::html::HTMLRenderer;
use crate::system::web::StyleSetter;
use crate::display::navigation::navigator::Navigator;

use crate::animation::*;
use crate::animation::physics::*;

use nalgebra::{Vector2, Vector3, zero};

fn create_scene(dim:Vector2<f32>) -> Scene<HTMLObject> {
    let mut scene : Scene<HTMLObject> = Scene::new();

    let width  = dim.x / 2.0;
    let height = dim.y / 2.0;

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
        object.set_position(Vector3::new(width * x, height * y, 0.0));
        let (r, g, b) = colors[i];
        let color = format!("rgb({}, {}, {})", r, g, b);
        object.dom.set_property_or_panic("background-color", color);
        scene.add(object);
    }

    scene
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_camera_navigation() {
    let renderer = HTMLRenderer::new("app").expect("Renderer couldn't be created");
    renderer.container.dom.set_property_or_panic("background-color", "black");

    let dimensions = renderer.dimensions();
    let scene = create_scene(dimensions);

    let mut camera  = Camera::perspective(90.0, dimensions.x / dimensions.y, 1.0, 1000.0);

    let x = dimensions.x / 2.0;
    let y = dimensions.y / 2.0;
    let z = y * camera.get_y_scale();
    camera.set_position(Vector3::new(x, y, z));

    let zoom_speed    = 6.0;
    let navigator     = Navigator::new(&renderer.container, camera.position(), zoom_speed);
    let mut navigator = navigator.expect("Couldn't create navigator");

    let mass           = 25.0;
    let velocity       = zero();
    let kinematics     = KinematicsProperties::new(camera.position(), velocity, zero(), mass);
    let coefficient    = 10000.0;
    let fixed_point    = camera.position();
    let spring         = SpringProperties::new(coefficient, fixed_point);
    let drag           = DragProperties::new(1000.0);
    let mut properties = PhysicsProperties::new(kinematics, spring, drag);
    let simulator      = PhysicsSimulator::new(camera.object.clone(), properties.clone());

    renderer.render(&mut camera, &scene);

    std::mem::forget(ContinuousTimeAnimator::new(move |_| {
        let _keep_alive = &simulator;
        let position = navigator.navigate(&mut camera);
        renderer.render(&mut camera, &scene);
        //camera.set_position(kinematics.position);
        properties.mod_spring(|spring| spring.set_fixed_point(position));
    }));
}
