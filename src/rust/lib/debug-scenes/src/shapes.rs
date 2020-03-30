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
use ensogl::display::shape::primitive::system::ShapeSystemDefinition;
use ensogl::display::shape::Var;
use ensogl::display::world::*;
use ensogl::system::web;
use graph::component::node;
use graph::component::node::Node;
use graph::component::node::WeakNode;
use graph::component::cursor;
use graph::component::cursor::Cursor;
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
use wasm_bindgen::JsCast;
use ensogl::display::scene;
use ensogl::display::scene::{Scene, MouseTarget};
use ensogl::gui::component::Component;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_shapes() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();
    init(&World::new(&web::get_html_element_by_id("root").unwrap()));
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


use ensogl::control::event_loop::RawAnimationLoop;
use ensogl::control::event_loop::AnimationLoop;
use ensogl::control::event_loop::TimeInfo;
use ensogl::control::event_loop::FixedFrameRateSampler;
use ensogl::animation::physics::inertia::DynInertiaSimulator;
use ensogl::data::OptVec;
use ensogl::display::object::Id;
use im_rc as im;


#[derive(Clone,CloneRef,Debug,Default)]
pub struct NodeSet {
    data : Rc<RefCell<HashMap<Id,WeakNode>>>
}

impl NodeSet {
    pub fn insert(&self, node:&Node) {
        self.data.borrow_mut().insert(node.id(),node.downgrade());
    }

    pub fn get(&self, id:Id) -> Option<Node> {
        self.data.borrow().get(&id).and_then(|t| t.upgrade())
    }
}


fn init(world: &World) {
    let scene  = world.scene();
    let camera = scene.camera();
    let screen = camera.screen();
    let navigator = Navigator::new(&scene,&camera);


    let node1 = Node::new();//&node_registry);
    let cursor = Cursor::new();

    world.add_child(&cursor);
    world.add_child(&node1);

    node1.mod_position(|t| {
        t.x += 200.0;
        t.y += 200.0;
    });

    let _nodes = vec![node1];

    web::body().set_style_or_panic("cursor","none");

    let mouse = &scene.mouse.frp;

    let node_set = NodeSet::default();

    frp! {
        mouse_down_position    = mouse.position.sample        (&mouse.on_down);
        selection_zero         = source::<Position>           ();
        selection_size_down    = mouse.position.map2          (&mouse_down_position,|m,n|{m-n});
        selection_size_if_down = selection_size_down.gate     (&mouse.is_down);
        selection_size_on_up   = selection_zero.sample        (&mouse.on_up);
        selection_size         = selection_size_if_down.merge (&selection_size_on_up);


        mouse_down_target      = mouse.on_down.map            (enclose!((scene) move |_| scene.mouse.target.get()));


        node_mouse_down = source::<Option<Node>> ();

        add_node = source::<()> ();
        new_node = add_node.map2(&mouse.position, enclose!((node_set,node_mouse_down,world) move |_,pos| {
            let node = Node::new();
            node_set.insert(&node);
            let ttt = node.events.mouse_down.map("foo",enclose!((node_mouse_down,node) move |_| {
                node_mouse_down.event.emit(Some(node.clone_ref()))
            }));

            world.add_child(&node);
            node.mod_position(|t| {
                t.x += pos.x as f32;
                t.y += pos.y as f32;
            });
            Some(node)
        }));

//        nodes_update = nodes.map2(&new_node, |node_set,new_node| {
//            new_node.for_each_ref(|node| {
//                node_set.vec.borrow_mut().insert(node.clone_ref());
//            })
//        });


        foo = node_mouse_down.map(|opt_node| {

        })


    }




    mouse.position.map("cursor_position", enclose!((cursor) move |p| {
        cursor.shape.borrow().as_ref().for_each(|shape| {
            shape.position.set(Vector2::new(p.x as f32,p.y as f32));
        })
    }));

    selection_size.map("cursor_size", enclose!((cursor) move |p| {
        cursor.shape.borrow().as_ref().for_each(|shape| {
            shape.selection_size.set(Vector2::new(p.x as f32, p.y as f32));
        })
    }));




    mouse_down_target.map("mouse_down_target", enclose!((scene) move |target| {
        match target {
            display::scene::Target::Background => {}
            display::scene::Target::Symbol {symbol_id, instance_id} => {
                scene.shapes.get_mouse_target(&(*instance_id as usize)).for_each(|target| {
                    target.mouse_down().for_each(|t| t.event.emit(()));
                })
            }
        }
        println!("SELECTING {:?}", target);
    }));


    let c: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |val| {
        let val = val.unchecked_into::<web_sys::KeyboardEvent>();
        let key = val.key();
        if      key == "n" {
            add_node.event.emit(());
        }
    }));
    web::document().add_event_listener_with_callback("keydown",c.as_ref().unchecked_ref()).unwrap();
    c.forget();






    let mut iter:i32 = 0;
    let mut time:i32 = 0;
    let mut was_rendered = false;
    let mut loader_hidden = false;

    let world_clone = world.clone_ref();
    world.on_frame(move |_| {
        let _keep_alive = &world_clone;
        let _keep_alive = &navigator;
        let _keep_alive = &_nodes;
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
