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
use graph::node;
use graph::node::Node;
use graph::node::NodeRegistry;
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
use im_rc as im;

#[derive(Debug,Default,Clone)]
pub struct NodeSet {
    vec : Rc<RefCell<OptVec<Node>>>
}


fn init(world: &World) {
    let scene  = world.scene();
    let camera = scene.camera();
    let screen = camera.screen();
    let navigator = Navigator::new(&scene,&camera);


    let node_shape_system             = ShapeSystem::new(world,&node::shape());
    let node_selection_buffer         = node_shape_system.add_input("selection" , 0.0);


    let pointer_shape_system          = ShapeSystem::new(world,&mouse_pointer());
    let pointer_position_buffer       = pointer_shape_system.add_input("position" , Vector2::<f32>::new(0.0,0.0));
    let pointer_selection_size_buffer = pointer_shape_system.add_input("selection_size" , Vector2::<f32>::new(0.0,0.0));



    let shape = scene.dom.shape().current();

    let pointer = Pointer::new(shape.width(), shape.height());

    pointer_shape_system.set_alignment(alignment::HorizontalAlignment::Left, alignment::VerticalAlignment::Bottom);

    scene.shapes.insert(TypeId::of::<Node>(),node_shape_system.clone());
    scene.shapes.insert(TypeId::of::<Pointer>(),pointer_shape_system.clone());


    let pointer_view = scene.views.new();
    scene.views.main.remove(&pointer_shape_system.symbol);
    pointer_view.add(&pointer_shape_system.symbol);

//    shape_scene.shape_system_map.insert(TypeId::of::<Node>(),node_shape_system.clone());

//    let animator_ref : Rc<RefCell<Option<AnimationLoop<FixedFrameRateSampler<Box<dyn Fn(TimeInfo)>>>>>> = default();
//    let animator = AnimationLoop::new(FixedFrameRateSampler::new(60.0,Box::new(enclose!((animator_ref) move |t:TimeInfo| {
//        if t.local > 1000.0 {
//            *animator_ref.borrow_mut() = None;
//        }
//        println!("{:?}",t)
//    })) as Box<dyn Fn(TimeInfo)>));
//    *animator_ref.borrow_mut() = Some(animator);



//    let nodes : Rc<RefCell<HashMap<usize,Node>>> = default();

    let node_registry = NodeRegistry::default();

    let node1 = Node::new(&node_registry);
    let node2 = Node::new(&node_registry);
    let node3 = Node::new(&node_registry);


    world.add_child(&pointer);



    world.add_child(&node1);
//    world.add_child(&node2);
//    world.add_child(&node3);

    node1.mod_position(|t| {
        t.x += 200.0;
        t.y += 200.0;
    });

//    node2.mod_position(|t| {
//        t.x += 300.0;
//        t.y += 300.0;
//    });
//
//    node3.mod_position(|t| {
//        t.x += 400.0;
//        t.y += 200.0;
//    });


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

        node1_selection         = source::<f32>           ();
        node1_selection_a       = source::<f32>           ();

        nodes = source::<NodeSet> ();
        add_node = source::<()> ();
        new_node = add_node.map2(&mouse.position, enclose!((world,node_registry) move |_,pos| {
            let node = Node::new(&node_registry);
            world.add_child(&node);
            node.mod_position(|t| {
                t.x += pos.x as f32;
                t.y += pos.y as f32;
            });
            Some(node)
        }));

        nodes_update = nodes.map2(&new_node, |node_set,new_node| {
            new_node.for_each_ref(|node| {
                node_set.vec.borrow_mut().insert(node.clone_ref());
            })
        });

        debug = node1_selection_a.map(|t| {println!("{:?}",t);})


    }

//    nodes_update.event.display_graphviz();

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


    let simulator = DynInertiaSimulator::<f32>::new(Box::new(move |t| {
        node1_selection_a.event.emit(t);
    }));


    node1_selection.map("node1_selection", move |value| {
        simulator.set_target_position(*value);
    });

    mouse_down_target.map("mouse_down_target", enclose!((node_registry,scene) move |target| {
        match target {
            display::scene::Target::Background => {}
            display::scene::Target::Symbol {symbol_id, instance_id} => {
                let br   = node_registry.map.borrow();
                let node = br.get(&(*instance_id as usize));
                node.for_each(|node| {
                    node.selection.event.emit(());
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
//        let _keep_alive = &animator_ref;
//        let _keep_alive = &simulator;

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
