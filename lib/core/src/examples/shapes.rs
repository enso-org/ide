#![allow(missing_docs)]

//use crate::prelude::*;

use crate::display::object::DisplayObjectOps;
use crate::display::symbol::geometry::sprite::Sprite;
use crate::display::symbol::geometry::sprite::SpriteSystem;
use crate::display::world::*;
use crate::system::web::set_stdout;

use nalgebra::Vector2;
use wasm_bindgen::prelude::*;



#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_shapes() {
    set_panic_hook();
    console_error_panic_hook::set_once();
    set_stdout();
    init(&WorldData::new("canvas"));
}

fn init(world: &World) {
    let sprite_system = SpriteSystem::new(world);
    let sprite = sprite_system.new_instance();
    sprite.set_bbox(Vector2::new(100.0,100.0));
    sprite.mod_position(|t| {
        t.x += 250.0;
        t.y += 100.0;
    });

    let mut iter:i32 = 0;
    let mut time:i32 = 0;
    world.on_frame(move |_| {
        on_frame(&mut time,&mut iter,&sprite,&sprite_system)
    }).forget();
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::many_single_char_names)]
pub fn on_frame
( _time         : &mut i32
, iter          : &mut i32
, _sprite1      : &Sprite
, sprite_system : &SpriteSystem) {
    *iter += 1;
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
