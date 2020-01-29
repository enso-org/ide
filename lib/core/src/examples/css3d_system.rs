#![allow(missing_docs)]

use crate::display::object::DisplayObject;
use crate::display::object::DisplayObjectOps;
use crate::display::symbol::geometry::Sprite;
use crate::display::symbol::geometry::SpriteSystem;
use crate::display::world::*;
use crate::prelude::*;
use crate::system::web::forward_panic_hook_to_console;
use crate::system::web::set_stdout;

use nalgebra::Vector2;
use nalgebra::Vector3;
use wasm_bindgen::prelude::*;
use crate::display::navigation::navigator::Navigator;

#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_css3d_system() {
    forward_panic_hook_to_console();
    set_stdout();
    init(&WorldData::new("canvas"));
}

fn init(world: &World) {
    let scene         = world.scene();
    let camera        = scene.camera();
    let screen        = camera.screen();
    let navigator     = Navigator::new(&scene, &camera).expect("Couldn't create navigator");
    let sprite_system = SpriteSystem::new();
//    let css3d_system  = Css3dSystem::new();
    world.add_child(&sprite_system);
//    world.add_child(&css3d_system);

    let mut sprites: Vec<Sprite> = default();
//    let mut css3d_objects: Vec<Css3dObject> = default();
    let count = 10;
    for i in 0 .. count {
        if i % 2 == 0 {
            let width = screen.width / count as f32;
            let height = screen.height;
            let dimensions = Vector2::new(width, screen.height);
            let x = i as f32;
            let sprite = sprite_system.new_instance();
            sprite.size().set(dimensions);
            sprite.mod_position(|t| *t = Vector3::new(width * x + width / 2.0, height / 2.0, 0.0));
            sprites.push(sprite);
        }
    }
    world.display_object().update();

    std::mem::forget(world);
    std::mem::forget(navigator);
    std::mem::forget(sprites);
}