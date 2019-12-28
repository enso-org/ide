use super::*;
use crate::set_stdout;
use crate::display::world::*;
use crate::prelude::*;
use nalgebra::{Vector2, Vector3, Matrix4};
use wasm_bindgen::prelude::*;
use basegl_system_web::{Logger, get_performance};
use web_sys::Performance;
use crate::display::object::DisplayObjectData;
use crate::display::object::DisplayObjectOps;
use crate::display::object::Modify;

use crate::display::symbol::geometry::sprite::Sprite;
use crate::display::symbol::geometry::sprite::SpriteSystem;



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
    for i in 0 .. count {
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

    if *ii < 1000i32 {
//            let count = 100;
//            if sprites.len() < 100_000 {
//                for _ in 0..count {
//                    let widget = make_widget(inst_scope);
//                    sprites.push(widget);
//                }
//            }

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
