#![allow(missing_docs)]

use wasm_bindgen::prelude::*;

use crate::display::camera::Camera2d;
use crate::system::web::dom::Scene;
use crate::system::web::dom::html::HTMLObject;
use crate::system::web::dom::html::HTMLRenderer;
use crate::control::EventLoop;
use crate::system::web::StyleSetter;
use crate::system::web::create_element;
use crate::system::web::get_webgl2_context;
use crate::system::web::set_stdout;
use crate::display::navigation::navigator::Navigator;
use wasm_bindgen::JsCast;

use crate::animation::animator::continuous::ContinuousAnimator;

use nalgebra::Vector2;
use nalgebra::Vector3;
use logger::Logger;
use crate::display::world::UniformScope;

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
        let (p_x, p_y) = positions[i];
        object.set_dimensions(width, height);
        object.set_position(Vector3::new(width * p_x, height * p_y, 0.0));
        let (c_r, c_g, c_b) = colors[i];
        let color = format!("rgb({}, {}, {})", c_r, c_g, c_b);
        object.dom.set_property_or_panic("background-color", color);
        scene.add(object);
    }

    scene
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_camera_navigation() {
    set_stdout();
    let renderer = HTMLRenderer::new("app").expect("Renderer couldn't be created");
    renderer.container.dom.set_property_or_panic("background-color", "black");

    let dimensions = renderer.dimensions();
    let scene = create_scene(dimensions);

    let logger     = Logger::new("camera_navigation");
    let canvas     = create_element("canvas").expect("Couldn't create Canvas element")
                   .dyn_into().expect("Couldn't convert Canvas element");
    let context    = get_webgl2_context(&canvas).expect("Couldn't get WebGL2 context");
    let variables  = UniformScope::new(logger.sub("global_variables"),&context);
    let mut camera = Camera2d::new(logger,&variables);
    camera.set_screen(dimensions.x, dimensions.y);
    camera.update();

    let y_scale = camera.projection_matrix().m11;
    let x = dimensions.x / 2.0;
    let y = dimensions.y / 2.0;
    let z = y * y_scale;
    camera.set_position(Vector3::new(x, y, z));

    let mut event_loop = EventLoop::new();

    let camera_clone = camera.clone();
    let navigator  = Navigator::new(&mut event_loop, &renderer.container, camera_clone);
    let navigator  = navigator.expect("Couldn't create navigator");

    let animator = ContinuousAnimator::new(&mut event_loop, move |_| {
        let _keep_alive = &navigator;
        renderer.render(&mut camera, &scene);
    });
    std::mem::forget(animator);
    std::mem::forget(event_loop);
}
