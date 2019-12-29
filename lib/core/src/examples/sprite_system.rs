#![allow(missing_docs)]

use crate::display::object::{DisplayObjectOps, DisplayObject};
use crate::display::symbol::geometry::sprite::Sprite;
use crate::display::symbol::geometry::sprite::SpriteSystem;
use crate::display::world::*;
use crate::prelude::*;
use crate::system::web::set_stdout;

use basegl_system_web::get_performance;
use nalgebra::Vector3;
use wasm_bindgen::prelude::*;
use web_sys::Performance;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_sprite_system() {
    set_panic_hook();
    console_error_panic_hook::set_once();
    set_stdout();
    init(&WorldData::new("canvas"));
}

fn init(world: &World) {
    let sprite_system = SpriteSystem::new(world);
    let sprite1 = sprite_system.new_instance();
    sprite1.mod_position(|t| t.y += 0.5);

    let mut sprites: Vec<Sprite> = default();
    let count = 100;
    for _ in 0 .. count {
        let sprite = sprite_system.new_instance();
        sprites.push(sprite);
    }

    let performance = get_performance().unwrap();
    let mut i:i32 = 0;
    world.on_frame(move |_| on_frame(&mut i,&sprite1,&mut sprites,&performance,&sprite_system)).forget();
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::many_single_char_names)]
pub fn on_frame(ii:&mut i32, sprite1:&Sprite, sprites:&mut Vec<Sprite>, performance:&Performance,sprite_system:&SpriteSystem) {
//        camera.mod_position(|p| {
//            p.x -= 0.1;
//            p.z += 1.0
//        });

    if *ii < 50i32 {
        sprite1.mod_position(|p| p.y += 0.5);
    }

    *ii += 1;

    let cycle_duration        = 200;
    let sprite_diff_per_cycle = 100;

    if *ii < cycle_duration {
        for _ in 0..sprite_diff_per_cycle {
            let sprite = sprite_system.new_instance();
            sprites.push(sprite);
        }
    } else if *ii < (cycle_duration * 2) {
        for _ in 0..sprite_diff_per_cycle {
            sprites.pop();
        }
    } else {
        *ii = 0;
        *sprites = default();
    }


    let t = (performance.now() / 1000.0) as f32;
    let length = sprites.len() as f32;
    for (i, sprite) in sprites.iter_mut().enumerate() {
        let i = i as f32;
        let d = (i / length - 0.5) * 2.0;

        let mut y = d;
        let r = (1.0 - y * y).sqrt();
        let mut x = (y * 100.0 + t).cos() * r;
        let mut z = (y * 100.0 + t).sin() * r;

        x += (y * 1.25 + t * 2.50).cos() * 0.5;
        y += (z * 1.25 + t * 2.00).cos() * 0.5;
        z += (x * 1.25 + t * 3.25).cos() * 0.5;
        sprite.set_position(Vector3::new(x * 50.0 + 200.0, y * 50.0 + 100.0, z * 50.0));
    }
    sprite_system.update();
}


pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
}
