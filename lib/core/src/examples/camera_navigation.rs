#![allow(missing_docs)]

use wasm_bindgen::prelude::*;

use crate::display::camera::Camera2d;
use crate::system::web::dom::html::HtmlScene;
use crate::system::web::dom::html::HtmlObject;
use crate::system::web::dom::html::HtmlRenderer;
use crate::control::EventLoop;
use crate::system::web::StyleSetter;
use crate::system::web::set_stdout;
use crate::display::navigation::navigator::Navigator;

use crate::animation::animator::continuous::ContinuousAnimator;

use nalgebra::Vector2;
use nalgebra::Vector3;
use logger::Logger;

fn create_scene(logger:Logger, dim:Vector2<f32>) -> HtmlScene {
    let mut scene = HtmlScene::new(logger.clone());

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
        let     object = HtmlObject::new(logger.clone(), "div");
        let mut object = object.expect("Couldn't create div");
        let (p_x, p_y) = positions[i];
        object.set_dimensions(width, height);
        object.set_position(Vector3::new(width * p_x, height * p_y, 0.0));
        let (c_r, c_g, c_b) = colors[i];
        let color = format!("rgb({}, {}, {})", c_r, c_g, c_b);
        object.dom.set_property_or_panic("background-color", color);
        scene.add_child(object);
    }

    scene
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_camera_navigation() {
    let logger     = Logger::new("camera_navigation");
    set_stdout();
    let renderer = HtmlRenderer::new("app").expect("Renderer couldn't be created");
    renderer.container().dom.set_property_or_panic("background-color", "black");

    let dimensions = renderer.dimensions();
    let scene = create_scene(logger.clone(), dimensions);

    let mut camera = Camera2d::new(logger,dimensions.x,dimensions.y);
    camera.update();

    let fovy_slope = camera.half_fovy_slope();
    let x = dimensions.x / 2.0;
    let y = dimensions.y / 2.0;
    let z = y / fovy_slope;
    camera.set_position(Vector3::new(x, y, z));

    let mut event_loop = EventLoop::new();

    let camera_clone = camera.clone();
    let navigator  = Navigator::new(&mut event_loop, renderer.container(), camera_clone);
    let navigator  = navigator.expect("Couldn't create navigator");

    let animator = ContinuousAnimator::new(&mut event_loop, move |_| {
        let _keep_alive = &navigator;
        renderer.render(&mut camera, &scene);
    });
    std::mem::forget(animator);
    std::mem::forget(event_loop);
}
