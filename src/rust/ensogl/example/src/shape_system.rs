//! Example scene showing simple usage of a shape system.

use ensogl_core::prelude::*;

use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::system::web;
use wasm_bindgen::prelude::*;
use ensogl_core::display::object::ObjectOps;
use ensogl_core::display::shape::ShapeSystem;
use ensogl_core::display::world::*;
use ensogl_core::display::shape::*;
use ensogl_core::data::color;
use js_sys::Math::sin;
use nalgebra::Vector2;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use js_sys::Math::random;


// ==============
// === Shapes ===
// ==============

/// The shape definition.
pub fn shape() -> AnyShape {
    let shape_width : Var<Pixels> = "input_size.x".into();
    let circle                   = Circle(shape_width / 3.0);
    let shape                    = circle;
    // let color : Var<Vector4>     = "input_colour".into();
    // let color : Var<color::Rgba> = color.into();
    // let shape                    = shape.fill(color);
    let shape      = shape.fill(color::Rgb::new(1.0,0.0,0.0));
    shape.into()
}


fn generate_data(item_count:usize) -> Vec<Vector3<f32>> {
    let mut data = Vec::with_capacity(item_count);
    for _ in 0..item_count {
        let x = random() * 1000.0;
        let y = random() * 1000.0;
        let z = random() * 10.0;
        data.push(Vector3::new(x as f32,y as f32,z as f32));
    }
    data
}

// fn example() -> Result<(), Box<dyn Error>> {
//     // Build the CSV reader and iterate over each record.
//     let mut rdr = csv::Reader::from_reader(io::stdin());
//     for result in rdr.records() {
//         // The iterator yields Result<StringRecord, Error>, so we check the
//         // error here.
//         let record = result?;
//         println!("{:?}", record);
//     }
//     Ok(())
// }

// ===================
// === Entry Point ===
// ===================

/// The example entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_shape_system() {
    web::forward_panic_hook_to_console();
    web::set_stdout();

    let world         = World::new(&web::get_html_element_by_id("root").unwrap());
    let scene         = world.scene();
    let camera        = scene.camera().clone_ref();
    let navigator     = Navigator::new(&scene,&camera);
    let sprite_system = ShapeSystem::new(&world,&shape());
    sprite_system.add_input("color", Vector4::<f32>::default());

    let performance = web::performance();

    let data_point_count = 1_000_000;
    let data = generate_data(data_point_count);

    let start = performance.now();

    // Actually slower than the below solution.
    // let sprites = sprite_system.new_instances(data_point_count);

    let mut sprites = Vec::default();
    sprites.resize_with(data_point_count, || sprite_system.new_instance());

    println!("Init. {}", performance.now() - start);
    for (sprite, data) in sprites.iter().zip(data) {
        sprite.size.set(Vector2::new(data.z,data.z));
        sprite.mod_position(|p| *p = Vector3(data.x,data.y,0.0));
    }
    println!("Data. {}", performance.now() - start);

    world.add_child(&sprite_system);
    world.keep_alive_forever();

    println!("Starting frames. {}", performance.now() - start);
    world.on_frame(move |time_info| {
        let _keep_alive = &sprites;
        let _keep_alive = &navigator;
        // println!("{:?}", time_info);
    }).forget();
}
