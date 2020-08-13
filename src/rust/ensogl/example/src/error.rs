#![allow(missing_docs)]

use ensogl_core::traits::*;

use ensogl_core::display::camera::Camera2d;
use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::display::symbol::geometry::Sprite;
use ensogl_core::display::symbol::geometry::SpriteSystem;
use ensogl_core::display::world::*;
use ensogl_core::prelude::*;
use ensogl_core::system::web::forward_panic_hook_to_console;
use ensogl_core::system::web::set_stdout;
use ensogl_core::system::web;
use ensogl_core::animation;
use nalgebra::Vector2;
use nalgebra::Vector3;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_error() {
    forward_panic_hook_to_console();
    set_stdout();

    let world         = World::new(&web::get_html_element_by_id("root").unwrap());
    let scene         = world.scene();
    let camera        = scene.camera().clone_ref();
    let navigator     = Navigator::new(&scene,&camera);
    let sprite_system = SpriteSystem::new(&world);

    let node   = sprite_system.new_instance();
    let root   = sprite_system.new_instance();
    let child1 = sprite_system.new_instance();
    let child2 = sprite_system.new_instance();


    node.size.set(Vector2::new(40.0,5.0));
    root.size.set(Vector2::new(15.0,15.0));
    child1.size.set(Vector2::new(10.0,10.0));
    child2.size.set(Vector2::new(7.0,7.0));
    root.mod_position(|t| *t = Vector3::new(0.0,20.0,0.0));
    child1.mod_position(|t| *t = Vector3::new(0.0,20.0,0.0));
    child2.mod_position(|t| *t = Vector3::new(20.0,20.0,0.0));

    world.add_child(&node);
    node.add_child(&root);
    root.add_child(&child1);
    child1.add_child(&child2);

    child2.unset_parent();
    child1.unset_parent();

    world.keep_alive_forever();

    let mut iter:i32 = 0;
    world.on_frame(move |time| {
        let _keep_alive = &node;
        let _keep_alive = &root;
        let _keep_alive = &child1;
        let _keep_alive = &child2;
        let _keep_alive = &navigator;
    }).forget();
}
