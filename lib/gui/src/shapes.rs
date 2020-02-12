#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use basegl::display::object::DisplayObject;
use basegl::display::object::DisplayObjectOps;
use basegl::display::symbol::geometry::Sprite;
use basegl::display::shape::primitive::system::ShapeSystem;
use basegl::display::world::*;
use basegl::system::web::set_stdout;
use basegl::system::web::set_stack_trace_limit;
use basegl::system::web::forward_panic_hook_to_console;
use basegl::display::shape::primitive::def::*;

use nalgebra::Vector2;
use wasm_bindgen::prelude::*;

//use basegl::display::navigation::navigator::Navigator;

use basegl::prelude::*;
use enso_frp::*;

use basegl::system::web;
use basegl::control::io::mouse2;
use basegl::control::io::mouse2::MouseManager;
use basegl::data::color::*;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_shapes() {
    forward_panic_hook_to_console();
    set_stdout();
    set_stack_trace_limit();
    init(&WorldData::new(&web::body()));
}


fn nodes1(world:&World) -> ShapeSystem {
    let node_radius = 40.0;
    let border_size = 10.0;
    let node   = Circle(node_radius);
    let border = Circle(node_radius + border_size);
    let node   = node.fill(Srgb::new(0.96,0.96,0.96));
    let border = border.fill(Srgba::new(0.0,0.0,0.0,0.06));

    let shadow1 = Circle(node_radius + border_size);
    let shadow1_color = Gradient::new()
        .add(0.0,Srgba::new(0.0,0.0,0.0,0.08).into_linear())
        .add(1.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear());
    let shadow1_color = DistanceGradient::new(shadow1_color).max_distance(border_size).slope(Slope::InvExponent(5.0));
    let shadow1       = shadow1.fill(shadow1_color);

    let shadow2 = Circle(node_radius + border_size);
    let shadow2_color = Gradient::new()
        .add(0.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear())
        .add(1.0,Srgba::new(0.0,0.0,0.0,0.08).into_linear());
    let shadow2_color = DistanceGradient::new(shadow2_color).max_distance(border_size).slope(Slope::Exponent(5.0));
    let shadow2       = shadow2.fill(shadow2_color);

    let loader_margin   = 0.0;
    let loader_outer    = Circle(node_radius + border_size - loader_margin);
    let loader_inner    = Circle(node_radius + loader_margin);
    let loader_section  = Angle("clamp(input_time/2000.0 - 1.0) * 1.99 * PI").rotate("(clamp(input_time/2000.0 - 1.0) * 1.99 * PI)/2.0");
    let loader_corner_1 = Circle(border_size/2.0).translate(0.0,45.0);
    let loader_corner_2 = loader_corner_1.rotate("clamp(input_time/2000.0 - 1.0) * 1.99 * PI");
    let loader = &loader_outer - &loader_inner;
    let loader = &loader * &loader_section;
    let loader = &loader + &loader_corner_1;
    let loader = &loader + &loader_corner_2;

    let loader = loader.fill(Srgba::new(0.22,0.83,0.54,1.0)).rotate("input_time/200.0");

    let out = border + loader + shadow1 + shadow2 + node;
    ShapeSystem::new(world,&out)
}

fn nodes2(world:&World) -> ShapeSystem {
    let node_radius = 40.0;
    let border_size = 20.0;
    let node   = Circle(node_radius);
    let border = Circle(node_radius + border_size);
    let node   = node.fill(Srgb::new(0.97,0.96,0.95));
//    let node   = node.fill(Srgb::new(0.26,0.69,0.99));
    let border = border.fill(Srgba::new(0.0,0.0,0.0,0.06));

    let bg   = Circle(node_radius*2.0);
    let bg   = bg.fill(Srgb::new(0.91,0.91,0.90));


    let shadow1 = Circle(node_radius + border_size);
    let shadow1_color = Gradient::new()
        .add(0.0,Srgba::new(0.0,0.0,0.0,0.08).into_linear())
        .add(1.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear());
    let shadow1_color = DistanceGradient::new(shadow1_color).max_distance(border_size).slope(Slope::InvExponent(5.0));
    let shadow1       = shadow1.fill(shadow1_color);

    let shadow2 = Circle(node_radius + border_size);
    let shadow2_color = Gradient::new()
        .add(0.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear())
        .add(1.0,Srgba::new(0.0,0.0,0.0,0.3).into_linear());
    let shadow2_color = DistanceGradient::new(shadow2_color).max_distance(border_size).slope(Slope::Exponent(4.0));
    let shadow2       = shadow2.fill(shadow2_color);

    let loader_margin   = 0.0;
    let loader_outer    = Circle(node_radius + border_size - loader_margin);
    let loader_inner    = Circle(node_radius + loader_margin);
    let loader_section  = Angle("clamp(input_time/2000.0 - 1.0) * 1.99 * PI").rotate("(clamp(input_time/2000.0 - 1.0) * 1.99 * PI)/2.0");
    let loader_corner_1 = Circle(border_size/2.0).translate(0.0,node_radius + border_size/2.0);
    let loader_corner_2 = loader_corner_1.rotate("clamp(input_time/2000.0 - 1.0) * 1.99 * PI");
    let loader = &loader_outer - &loader_inner;
    let loader = &loader * &loader_section;
    let loader = &loader + &loader_corner_1;
    let loader = &loader + &loader_corner_2;

    let loader = loader.fill(Srgba::new(0.22,0.83,0.54,1.0)).rotate("input_time/200.0");

    let corner_radius1 = 2.0;
    let corner_radius2 = 1.5;
    let corner_radius3 = 1.0;
    let width_diff    = 3.0 * corner_radius1;
    let border_size   = 2.0;
    let rect1 = RoundedRectByCorner(32.0,16.0,corner_radius1,corner_radius1,corner_radius1,corner_radius1).fill(Srgba::new(0.26, 0.69, 0.99, 1.00));
    let rect2 = RoundedRectByCorner(32.0-width_diff,16.0,corner_radius2,corner_radius2,corner_radius2,corner_radius2).translate(0.0,6.0);
    let rect2 = rect2 - rect1.translate(0.0,border_size);
    let rect2 = rect2.fill(Srgba::new(0.26, 0.69, 0.99, 0.6));

    let rect3 = RoundedRectByCorner(32.0-2.0*width_diff,16.0,corner_radius3,corner_radius3,corner_radius3,corner_radius3).translate(0.0,10.0);
    let rect3 = rect3 - rect1.translate(0.0,border_size + 6.0);
    let rect3 = rect3.fill(Srgba::new(0.26, 0.69, 0.99, 0.4));
//    let loader = loader.fill(Srgb::new(0.91,0.91,0.90));

//    let out = bg + loader + shadow2 + node + front;

//    let out = rect3 + rect2 + rect1 ;
//    let out = out.scale(2.0);

    let rect1 = Rect(1.0,4.0).fill(Srgba::new(1.0, 0.0, 0.0, 1.00));

    let rect2 = rect1.translate(1.0,5.0);

//    let circle1 = Circle(node_radius + border_size);



    let out = rect1 + rect2;

//    let c1   = Circle(node_radius).fill(Srgba::new(1.0,0.0,0.0,0.5));
//    let c2   = c1.translate(20.0,0.0).fill(Srgba::new(0.0,1.0,0.0,0.5));
//
//    let bg   = Circle(node_radius*2.0).translate(0.0,70.0);
//    let bg   = bg.fill(Srgb::new(1.0,1.0,1.0));
//
//    let out = bg + shadow2;

    ShapeSystem::new(world,&out)
}


fn init(world: &World) {
    let scene  = world.scene();
    let camera = scene.camera();
    let screen = camera.screen();


    let shape_system = nodes2(world);




    let sprite = shape_system.new_instance();
    sprite.size().set(Vector2::new(200.0,200.0));
    sprite.mod_position(|t| {
        t.x += screen.width / 2.0;
        t.y += screen.height / 2.0;
    });

    let sprite_2 = shape_system.new_instance();
    sprite_2.size().set(Vector2::new(200.0,200.0));
    sprite_2.mod_position(|t| {
        t.x += screen.width / 2.0 + 5.0;
        t.y += screen.height / 2.0 + 20.0;
    });

    let sprite2 = sprite.clone();


    world.add_child(&shape_system);
//
//    let out = frp_test(Box::new(move|x:f32,y:f32| {
//        sprite2.set_position(Vector3::new(x,y,0.0));
//    }));

    let mut iter:i32 = 0;
    let mut time:i32 = 0;
    let mut was_rendered = false;
    let mut loader_hidden = false;
    world.on_frame(move |_| {
        let _keep_alive = &sprite;
//        let _keep_alive = &sprite_2;
//        let _keep_alive = &out;
        on_frame(&mut time,&mut iter,&sprite,&shape_system);
        if was_rendered && !loader_hidden {
            web::get_element_by_id("loader").map(|t| {
                t.parent_node().map(|p| {
                    p.remove_child(&t).unwrap()
                })
            }).ok();
            loader_hidden = true;
        }
        was_rendered = true;
    }).forget();
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::many_single_char_names)]
pub fn on_frame
( _time        : &mut i32
, iter         : &mut i32
, _sprite1     : &Sprite
, shape_system : &ShapeSystem) {
    *iter += 1;
    shape_system.display_object().update();
}



// ================
// === FRP Test ===
// ================

#[allow(unused_variables)]
pub fn frp_test (callback: Box<dyn Fn(f32,f32)>) -> MouseManager {
    let document        = web::document().unwrap();
    let mouse_manager   = MouseManager::new(&document);
    let mouse           = Mouse::new();

    frp! {
        mouse_down_position    = mouse.position.sample       (&mouse.down);
        mouse_position_if_down = mouse.position.gate         (&mouse.is_down);
        final_position_ref     = recursive::<Position>       ();
        pos_diff_on_down       = mouse_down_position.map2    (&final_position_ref,|m,f|{m-f});
        final_position         = mouse_position_if_down.map2 (&pos_diff_on_down  ,|m,f|{m-f});
        debug                  = final_position.sample       (&mouse.position);
    }
    final_position_ref.initialize(&final_position);

    // final_position.event.display_graphviz();

//    trace("X" , &debug.event);

//    final_position.map("foo",move|p| {callback(p.x as f32,-p.y as f32)});

    let target = mouse.position.event.clone_ref();
    let handle = mouse_manager.on_move.add(move |event:&mouse2::event::OnMove| {
        target.emit(Position::new(event.client_x(),event.client_y()));
    });
    handle.forget();

    let target = mouse.down.event.clone_ref();
    let handle = mouse_manager.on_down.add(move |event:&mouse2::event::OnDown| {
        target.emit(());
    });
    handle.forget();

    let target = mouse.up.event.clone_ref();
    let handle = mouse_manager.on_up.add(move |event:&mouse2::event::OnUp| {
        target.emit(());
    });
    handle.forget();

    mouse_manager
}
