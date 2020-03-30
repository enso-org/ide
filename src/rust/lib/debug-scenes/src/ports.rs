#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use ensogl::traits::*;
use ensogl::prelude::*;
use ensogl::data::color::*;
use ensogl::display::shape::*;
use ensogl::display::shape::compound::port::{Specification, Port, Direction};
use ensogl::display::shape::primitive::system::ShapeSystem;
use ensogl::display::symbol::geometry::Sprite;
use ensogl::display::world::*;
use ensogl::math::topology::unit::AngleOps;
use ensogl::system::web;
use nalgebra::Vector2;
use wasm_bindgen::prelude::*;
use ensogl::display::object::Node;
use ensogl::display::object::Object;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_ports() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    init(&WorldData::new(&web::get_html_element_by_id("root").unwrap()));
}

fn node(node_radius:f32, colour:Srgba<f32>) -> AnyShape {
    let node   = Circle(node_radius.px());//    let border = Circle((node_radius + border_size).px());
    let node   = node.fill(colour);
    let out    = node;
    out.into()
}

fn init(world: &World) {
    let scene  = world.scene();
    let camera = scene.camera();
    let screen = camera.screen();

    let node_radius = 60.0 ;
    let port_height = 30.0;

    let parent_node = Node::new(Logger::new("parent"));
    parent_node.mod_position(|t| {
        t.x += screen.width / 2.0;
        t.y += screen.height / 3.0;
    });
    parent_node.update();

    let node_shape = node(node_radius, Srgba::new(0.57,0.56,0.55,0.5));
    let node_shape_system =     ShapeSystem::new(world, &node_shape);
    let node_sprite = node_shape_system.new_instance();
    node_sprite.size().set(Vector2::new(200.0, 200.0));
    node_sprite.set_position(parent_node.global_position());

    let node2_shape = node(node_radius + port_height,Srgba::new(0.77,0.76,0.75,0.5));
    let node2_shape_system =     ShapeSystem::new(world, &node2_shape);
    let node2_sprite = node2_shape_system.new_instance();
    node2_sprite.size().set(Vector2::new(200.0, 200.0));
    node2_sprite.set_position(parent_node.global_position());

    let port_spec = Specification{
        height: port_height,
        width: Angle::from(25.0),
        inner_radius: node_radius,
        direction: Direction::Outwards,
        location: 90.0_f32.deg(),
        color: Srgb::new(51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0 ),
    };

    let mut port_1 = Port::new(port_spec, &world, &parent_node);

    let port_spec = Specification{
        height: port_height,
        width: Angle::from(25.0),
        inner_radius: node_radius,
        direction: Direction::Inwards,
        location: 270.0_f32.deg(),
        color: Srgb::new(51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0 ),
    };

    let mut port_2 = Port::new(port_spec, &world, &parent_node);

    world.add_child(&port_1);
    world.add_child(&port_2);
    world.add_child(&node_shape_system);
    world.add_child(&node2_shape_system);

    let mut iter:i32 = 0;
    let mut time:i32 = 0;
    world.on_frame(move |_| {
        iter +=1;
        port_1.mod_specification(|s|{
            s.location = Angle::from(s.location.value + 1.0);
        });
        port_2.mod_specification(|s|{
            s.location = Angle::from(s.location.value + 1.0);
        });

        parent_node.update();
        port_1.update();
        port_2.update();
        on_frame(&mut time,&mut iter,&node_sprite,&node_shape_system);
        on_frame(&mut time,&mut iter,&node2_sprite,&node2_shape_system);

    }).forget();
}



#[allow(clippy::too_many_arguments)]
#[allow(clippy::many_single_char_names)]
pub fn on_frame
( _time        : &mut i32
  , _iter         : &mut i32
  , sprite     : &Sprite
  , shape_system : &ShapeSystem) {
    // sprite.mod_rotation(|r| r.z += 0.01);
    shape_system.display_object().update();
}
