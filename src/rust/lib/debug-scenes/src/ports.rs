#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use ensogl::traits::*;

use ensogl::data::color::*;
use ensogl::display::shape::*;
use ensogl::display::shape::compound::port::{PortSpecification, Port, PortDirection};
use ensogl::display::shape::primitive::system::ShapeSystem;
use ensogl::display::symbol::geometry::Sprite;
use ensogl::display::world::*;
use ensogl::math::topology::unit::AngleOps;
use ensogl::system::web;
use nalgebra::Vector2;
use wasm_bindgen::prelude::*;



#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_ports() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    init(&WorldData::new(&web::get_html_element_by_id("root").unwrap()));
}


fn init(world: &World) {
    let scene  = world.scene();
    let camera = scene.camera();
    let screen = camera.screen();

    let port_spec = PortSpecification{
        height: 30.0,
        width: Angle::from(45.0),
        inner_radius: 80.0,
        direction: PortDirection::Outwards,
        location: 90.0_f32.deg(),
        color: Srgb::new(51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0 ),
    };

    let port_shape_1 = Port::new(port_spec);

    let port_spec = PortSpecification{
        height: 30.0,
        width: Angle::from(45.0),
        inner_radius: 80.0,
        direction: PortDirection::Inwards,
        location: 270.0_f32.deg(),
        color: Srgb::new(51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0 ),
    };

    let port_shape_2 = Port::new(port_spec);

    let shape_system =     ShapeSystem::new(world,&port_shape_1);
    let sprite = shape_system.new_instance();
    sprite.size().set(Vector2::new(500.0,500.0));
    sprite.mod_position(|t| {
        t.x += screen.width / 2.0;
        t.y += screen.height / 2.0;
    });

    let shape_system_2 =     ShapeSystem::new(world,&port_shape_2);
    let sprite_2 = shape_system_2.new_instance();
    sprite_2.size().set(Vector2::new(500.0,500.0));
    sprite_2.mod_position(|t| {
        t.x += screen.width / 2.0;
        t.y += screen.height / 2.0;
    });


    world.add_child(&shape_system);
    world.add_child(&shape_system_2);

    let mut iter:i32 = 0;
    let mut time:i32 = 0;
    world.on_frame(move |_| {
        iter +=1;
        on_frame(&mut time,&mut iter,&sprite,&shape_system);
        on_frame(&mut time,&mut iter,&sprite_2,&shape_system_2);

    }).forget();
}


#[allow(clippy::too_many_arguments)]
#[allow(clippy::many_single_char_names)]
pub fn on_frame
( _time        : &mut i32
  , _iter         : &mut i32
  , sprite     : &Sprite
  , shape_system : &ShapeSystem) {
    sprite.mod_rotation(|r| r.z += 0.01);
    shape_system.display_object().update();
}
