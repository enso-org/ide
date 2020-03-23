#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use ensogl::prelude::*;
use ensogl::traits::*;

use ensogl::data::color::*;
use ensogl::display;
use ensogl::display::Sprite;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::system::ShapeSystem;
use ensogl::display::shape::Var;
use ensogl::display::world::*;
use ensogl::system::web;
use graph::node::Node;
use nalgebra::Vector2;
use shapely::shared;
use std::any::TypeId;
use wasm_bindgen::prelude::*;
use ensogl::control::io::mouse::MouseManager;
use enso_frp::{frp, Position};
use enso_frp::Mouse;
use ensogl::control::io::mouse;
use enso_frp::core::node::class::EventEmitterPoly;
use ensogl_system_web::StyleSetter;
use ensogl::display::layout::alignment;



#[derive(Debug,Clone)]
pub struct Pointer {
    logger         : Logger,
    display_object : display::object::Node,
    sprite         : Rc<CloneCell<Option<Sprite>>>,
}

impl CloneRef for Pointer {}

impl Pointer {
    pub fn new(width:f32,height:f32) -> Self {
        let logger = Logger::new("mouse.pointer");
        let sprite : Rc<CloneCell<Option<Sprite>>> = default();
        let display_object      = display::object::Node::new(&logger);
        let display_object_weak = display_object.downgrade();

        display_object.set_on_show_with(enclose!((sprite) move |scene| {
            let type_id      = TypeId::of::<Pointer>();
            let shape_system = scene.shapes.get(&type_id).unwrap();
            let new_sprite   = shape_system.new_instance();
            display_object_weak.upgrade().for_each(|t| t.add_child(&new_sprite));
            new_sprite.size().set(Vector2::new(width,height));
            sprite.set(Some(new_sprite));
        }));

        display_object.set_on_hide_with(enclose!((sprite) move |_| {
            sprite.set(None);
        }));

        Self {logger,sprite,display_object}
    }
}

impl<'t> From<&'t Pointer> for &'t display::object::Node {
    fn from(ptr:&'t Pointer) -> Self {
        &ptr.display_object
    }
}



#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_shapes() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    init(&World::new(&web::get_html_element_by_id("root").unwrap()));
}

pub mod icons {
    use super::*;

    pub fn history() -> AnyShape {
        let radius_diff    = 0.5.px();
        let corners_radius = 2.0.px();
        let width_diff     = &corners_radius * 3.0;
        let offset         = 2.px();
        let width          = 32.px();
        let height         = 16.px();
        let persp_diff1    = 6.px();

        let width2          = &width  - &width_diff;
        let width3          = &width2 - &width_diff;
        let corners_radius2 = &corners_radius  - &radius_diff;
        let corners_radius3 = &corners_radius2 - &radius_diff;
        let persp_diff2     = &persp_diff1 * 2.0;

        let rect1 = Rect((&width ,&height)).corners_radius(&corners_radius);
        let rect2 = Rect((&width2,&height)).corners_radius(&corners_radius2).translate_y(&persp_diff1);
        let rect3 = Rect((&width3,&height)).corners_radius(&corners_radius3).translate_y(&persp_diff2);

        let rect3 = rect3 - rect2.translate_y(&offset);
        let rect2 = rect2 - rect1.translate_y(&offset);

        let rect1 = rect1.fill(Srgba::new(0.26, 0.69, 0.99, 1.00));
        let rect2 = rect2.fill(Srgba::new(0.26, 0.69, 0.99, 0.6));
        let rect3 = rect3.fill(Srgba::new(0.26, 0.69, 0.99, 0.4));

        let icon = (rect3 + rect2 + rect1).translate_y(-persp_diff2/2.0);
        icon.into()
    }
}

fn ring_angle<R,W,A>(inner_radius:R, width:W, angle:A) -> AnyShape
where R : Into<Var<Distance<Pixels>>>,
      W : Into<Var<Distance<Pixels>>>,
      A : Into<Var<Angle<Radians>>> {
    let inner_radius = inner_radius.into();
    let width        = width.into();
    let angle        = angle.into();

    let angle2  = &angle / 2.0;
    let radius  = &width / 2.0;
    let inner   = Circle(&inner_radius);
    let outer   = Circle(&inner_radius + &width);
    let section = Plane().cut_angle(&angle);
    let corner1 = Circle(&radius).translate_y(inner_radius + radius);
    let corner2 = corner1.rotate(&angle2);
    let corner1 = corner1.rotate(-&angle2);
    let ring    = &outer - &inner;
    let pie     = &ring * &section;
    let out     = &pie + &corner1 + &corner2;
    let out     = out.fill(Srgba::new(0.9,0.9,0.9,1.0));
    out.into()
}

fn nodes_shape() -> AnyShape {
    let node_radius = 32.0;
    let border_size = 16.0;

    let node = Circle(node_radius.px());
    let node = node.fill(Srgb::new(0.97,0.96,0.95));
    let bg   = Circle((node_radius*2.0).px());
    let bg   = bg.fill(Srgb::new(0.91,0.91,0.90));

    let shadow2 = Circle((node_radius + border_size).px());
    let shadow2_color = LinearGradient::new()
        .add(0.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear())
        .add(1.0,Srgba::new(0.0,0.0,0.0,0.14).into_linear());
    let shadow2_color = SdfSampler::new(shadow2_color).max_distance(border_size).slope(Slope::Exponent(4.0));
    let shadow2       = shadow2.fill(shadow2_color);

    let selection = Circle((node_radius + border_size).px() * "input_selection");
    let selection = selection.fill(Srgba::new(0.22,0.83,0.54,1.0));

    let loader_angle : Var<Angle<Radians>> = "Radians(clamp(input_time/2000.0 - 1.0) * 1.99 * PI)".into();
    let loader_angle2 = &loader_angle / 2.0;
    let loader        = ring_angle((node_radius).px(), (border_size).px(), loader_angle);
    let loader        = loader.rotate(loader_angle2);
    let loader        = loader.rotate("Radians(input_time/200.0)");
    let icon          = icons::history();
    let out           = loader + selection + shadow2 + node + icon;
    out.into()
}

fn mouse_pointer() -> AnyShape {
    let radius  = 10.px();
    let side    = &radius * 2.0;
    let width   = Var::<Distance<Pixels>>::from("input_selection_size.x");
    let height  = Var::<Distance<Pixels>>::from("input_selection_size.y");
    let pointer = Rect((&side + width.abs(),&side + height.abs()))
        .corners_radius(radius)
        .translate((-&width/2.0, -&height/2.0))
        .translate(("input_position.x","input_position.y"))
        .fill(Srgba::new(0.0,0.0,0.0,0.3));
    pointer.into()
}

fn init(world: &World) {
    let scene  = world.scene();
    let camera = scene.camera();
    let screen = camera.screen();
    let navigator = Navigator::new(&scene,&camera);


    let node_shape_system             = ShapeSystem::new(world,&nodes_shape());
    let pointer_shape_system          = ShapeSystem::new(world,&mouse_pointer());
    let pointer_position_buffer       = pointer_shape_system.add_input("position" , Vector2::<f32>::new(0.0,0.0));
    let pointer_selection_size_buffer = pointer_shape_system.add_input("selection_size" , Vector2::<f32>::new(0.0,0.0));

    let node_selection_buffer         = node_shape_system.add_input("selection" , 0.0);


    let shape = scene.dom.shape().current();

    let pointer = Pointer::new(shape.width(), shape.height());

    pointer_shape_system.set_alignment(alignment::HorizontalAlignment::Left, alignment::VerticalAlignment::Bottom);

    scene.shapes.insert(TypeId::of::<Node>(),node_shape_system.clone());
    scene.shapes.insert(TypeId::of::<Pointer>(),pointer_shape_system.clone());


    let pointer_view = scene.views.new();
    scene.views.main.remove(&pointer_shape_system.symbol);
    pointer_view.add(&pointer_shape_system.symbol);

//    shape_scene.shape_system_map.insert(TypeId::of::<Node>(),node_shape_system.clone());


    let node1 = Node::new();
    let node2 = Node::new();
    let node3 = Node::new();

    world.add_child(&pointer);



    world.add_child(&node1);
    world.add_child(&node2);
    world.add_child(&node3);

    node1.mod_position(|t| {
        t.x += 200.0;
        t.y += 200.0;
    });

    node2.mod_position(|t| {
        t.x += 300.0;
        t.y += 300.0;
    });

    node3.mod_position(|t| {
        t.x += 400.0;
        t.y += 200.0;
    });


    let nodes = vec![node1,node2,node3];


    world.add_child(&node_shape_system);
    world.add_child(&pointer_shape_system);




    web::body().set_style_or_panic("cursor","none");

    let mouse = &scene.mouse.frp;

    frp! {
        mouse_down_position    = mouse.position.sample        (&mouse.on_down);
        selection_zero         = source::<Position>           ();
        selection_size_down    = mouse.position.map2          (&mouse_down_position,|m,n|{m-n});
        selection_size_if_down = selection_size_down.gate     (&mouse.is_down);
        selection_size_on_up   = selection_zero.sample        (&mouse.on_up);
        selection_size         = selection_size_if_down.merge (&selection_size_on_up);


        mouse_down_target      = mouse.on_down.map            (enclose!((scene) move |_| scene.mouse.target.get()));
//        final_position_ref     = recursive::<Position>       ();
//        pos_diff_on_down       = mouse_down_position.map2    (&final_position_ref,|m,f|{m-f});
//        final_position         = mouse_position_if_down.map2 (&pos_diff_on_down  ,|m,f|{m-f});
//        debug                  = final_position.sample       (&mouse.position);

//        debug = mouse_down_target.map(|t| {println!("{:?}",t);})
    }

    mouse.position.map("pointer_position", enclose!((pointer) move |p| {
        pointer.sprite.get().for_each(|sprite| {
            let pointer_position = pointer_position_buffer.at(sprite.instance_id());
            pointer_position.set(Vector2::new(p.x as f32,p.y as f32));
        })
    }));

    selection_size.map("pointer_size", enclose!((pointer) move |p| {
        pointer.sprite.get().for_each(|sprite| {
            let pointer_size = pointer_selection_size_buffer.at(sprite.instance_id());
            pointer_size.set(Vector2::new(p.x as f32, p.y as f32));
        })
    }));


    mouse_down_target.map("mouse_down_target", enclose!((scene) move |target| {
        match target {
            display::scene::Target::Background => {}
            display::scene::Target::Symbol {symbol_id, instance_id} => {
                // FIXME
                // 1. The following line is unsound. It creates a new sprite which when gets dropped, is removed from buffers (!)
                // 2. We have not checked if the selection was node sprite.
                // let sprite = node_shape_system.from_instance_id(*instance_id as usize); // FIXME conversion
            }
        }
        println!("SELECTING {:?}", target);
    }));









    let mut iter:i32 = 0;
    let mut time:i32 = 0;
    let mut was_rendered = false;
    let mut loader_hidden = false;
    let mut i = 200;

    let world_clone = world.clone_ref();
    world.on_frame(move |_| {
        i -= 1;
        if i == 0 {
//            nodes[1].unset_parent();
//            node_shape_system.set_shape(&node_shape2);
        }
//        let _keep_alive = &sprite;
        let _keep_alive = &world_clone;
        let _keep_alive = &navigator;
        let _keep_alive = &nodes;
        let _keep_alive = &pointer_view;

//        let _keep_alive = &sprite_2;
//        let _keep_alive = &out;
        on_frame(&mut time,&mut iter);
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
, iter         : &mut i32) {
    *iter += 1;
}



// ================
// === FRP Test ===
// ================

//#[allow(unused_variables)]
//pub fn frp_test (callback: Box<dyn Fn(f32,f32)>) -> MouseManager {
//    let document        = web::document();
//    let mouse_manager   = MouseManager::new(&document);
//    let mouse           = Mouse::new();
//
//    frp! {
//        mouse_down_position    = mouse.position.sample       (&mouse.on_down);
//        mouse_position_if_down = mouse.position.gate         (&mouse.is_down);
//        final_position_ref     = recursive::<Position>       ();
//        pos_diff_on_down       = mouse_down_position.map2    (&final_position_ref,|m,f|{m-f});
//        final_position         = mouse_position_if_down.map2 (&pos_diff_on_down  ,|m,f|{m-f});
//        debug                  = final_position.sample       (&mouse.position);
//    }
//    final_position_ref.initialize(&final_position);
//
//    // final_position.event.display_graphviz();
//
////    trace("X" , &debug.event);
//
////    final_position.map("foo",move|p| {callback(p.x as f32,-p.y as f32)});
//
//    let target = mouse.position.event.clone_ref();
//    let handle = mouse_manager.on_move.add(move |event:&mouse::OnMove| {
//        target.emit(Position::new(event.client_x(),event.client_y()));
//    });
//    handle.forget();
//
//    let target = mouse.on_down.event.clone_ref();
//    let handle = mouse_manager.on_down.add(move |event:&mouse::OnDown| {
//        target.emit(());
//    });
//    handle.forget();
//
//    let target = mouse.on_up.event.clone_ref();
//    let handle = mouse_manager.on_up.add(move |event:&mouse::OnUp| {
//        target.emit(());
//    });
//    handle.forget();
//
//    mouse_manager
//}
