#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

#![feature(associated_type_defaults)]
#![feature(drain_filter)]
#![feature(overlapping_marker_traits)]
#![feature(specialization)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(unboxed_closures)]
#![feature(weak_into_raw)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]


#[warn(missing_docs)]
pub mod component;

/// Common types and functions usable in all modules of this crate.
pub mod prelude {
    pub use ensogl::prelude::*;
}

use ensogl::prelude::*;
use ensogl::traits::*;

use ensogl::display;
use ensogl::display::world::*;
use ensogl::system::web;
use crate::component::node::Node;
use crate::component::node::WeakNode;
use crate::component::cursor::Cursor;
use nalgebra::Vector2;
use enso_frp as frp;
use enso_frp::{frp, Position};
use enso_frp::core::node::class::EventEmitterPoly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use ensogl::display::object::Id;
use ensogl::system::web::StyleSetter;




#[derive(Clone,CloneRef,Debug,Default)]
pub struct NodeSet {
    data : Rc<RefCell<HashMap<Id,Node>>>
}

impl NodeSet {
    pub fn borrow(&self) -> Ref<HashMap<Id,Node>> {
        self.data.borrow()
    }

    pub fn take(&self) -> HashMap<Id,Node> {
        mem::take(&mut *self.data.borrow_mut())
    }

    pub fn insert(&self, node:Node) {
        self.data.borrow_mut().insert(node.id(),node);
    }

    pub fn remove(&self, node:&Node) {
        self.data.borrow_mut().remove(&node.id());
    }

    pub fn contains(&self, node:&Node) -> bool {
        self.get(node.id()).is_some()
    }

    pub fn get(&self, id:Id) -> Option<Node> {
        self.data.borrow().get(&id).map(|t| t.clone_ref())
    }

    pub fn clear(&self) {
        self.data.borrow_mut().clear();
    }
}



#[derive(Clone,CloneRef,Debug,Default)]
pub struct WeakNodeSet {
    data : Rc<RefCell<HashMap<Id,WeakNode>>>
}

impl WeakNodeSet {
    pub fn borrow(&self) -> Ref<HashMap<Id,WeakNode>> {
        self.data.borrow()
    }

    pub fn take(&self) -> HashMap<Id,WeakNode> {
        mem::take(&mut *self.data.borrow_mut())
    }

    pub fn for_each_taken<F:Fn(Node)>(&self,f:F) {
        self.take().into_iter().for_each(|(_,node)| { node.upgrade().for_each(|n| f(n)) })
    }

    pub fn insert(&self, node:&Node) {
        self.data.borrow_mut().insert(node.id(),node.downgrade());
    }

    pub fn contains(&self, node:&Node) -> bool {
        self.get(node.id()).is_some()
    }

    pub fn get(&self, id:Id) -> Option<Node> {
        self.data.borrow().get(&id).and_then(|t| t.upgrade())
    }
}


#[derive(Clone,CloneRef,Debug,Default,Shrinkwrap)]
pub struct WeakNodeSelectionSet {
    data : WeakNodeSet
}

impl WeakNodeSelectionSet {
    pub fn deselect_all(&self) {
        self.for_each_taken(|node| node.events.deselect.event.emit(()));
    }
}


#[derive(Debug)]
pub struct Events {
    pub add_node_under_cursor : frp::Dynamic<()>,
    pub add_node_at           : frp::Dynamic<Position>,
    pub remove_selected_nodes : frp::Dynamic<()>,
    pub clear_graph           : frp::Dynamic<()>,
}

impl Default for Events {
    fn default() -> Self {
        frp! {
            add_node_under_cursor = source::<()>       ();
            add_node_at           = source::<Position> ();
            remove_selected_nodes = source::<()>       ();
            clear_graph           = source::<()>       ();
        }
        Self {add_node_under_cursor,add_node_at,remove_selected_nodes,clear_graph}
    }
}

#[derive(Debug)]
pub struct GraphEditor {
    pub events         : Events,
    pub selected_nodes : WeakNodeSelectionSet,
    pub display_object : display::object::Node,
}

impl GraphEditor {
    pub fn new(world: &World) -> Self {
        let scene  = world.scene();
        let cursor = Cursor::new();
        web::body().set_style_or_panic("cursor","none");
        world.add_child(&cursor);

        let display_object = display::object::Node::new(Logger::new("GraphEditor"));

        let events = Events::default();
    //    web::body().set_style_or_panic("cursor","none");

        let mouse = &scene.mouse.frp;

        let node_set = NodeSet::default();

        let selected_nodes = WeakNodeSelectionSet::default();

        let selected_nodes2 = selected_nodes.clone_ref();


        frp! {
            on_node_press            = source::<Option<WeakNode>> ();
            on_node_press_bool       = on_node_press.map(|_| true);
            on_mouse_up_bool         = mouse.on_up.map(|_| false);
            node_is_pressed          = on_node_press_bool.merge(&on_mouse_up_bool);
            node_was_pressed         = node_is_pressed.previous();
            on_release               = mouse.on_up.gate(&node_was_pressed);
            mouse_pos_on_node_press  = mouse.position.sample(&on_node_press);
            mouse_pos_on_release     = mouse.position.sample(&on_release);
            node_should_select       = mouse_pos_on_release.map2(&mouse_pos_on_node_press,|p1,p2| p1==p2);
            on_node_release          = on_node_press.sample(&on_release);
            node_select              = on_node_release.gate(&node_should_select);
       }

       frp! {

            on_bg_press              = source::<()> ();
            on_bg_press_bool         = on_bg_press.map(|_| true);
            bg_is_pressed            = on_bg_press_bool.merge(&on_mouse_up_bool);
            bg_was_pressed           = bg_is_pressed.previous();
            on_release               = mouse.on_up.gate(&bg_was_pressed);
            mouse_pos_on_bg_press    = mouse.position.sample(&on_bg_press);
            mouse_pos_on_release     = mouse.position.sample(&on_release);
            bg_should_select         = mouse_pos_on_release.map2(&mouse_pos_on_bg_press,|p1,p2| p1==p2);
            on_bg_release            = on_bg_press.sample(&on_release);
            bg_select                = on_bg_release.gate(&bg_should_select);

            _bg_selection = bg_select.map(move |_| {
                selected_nodes2.deselect_all();
            });
            _debug = bg_select.map(|t| println!(">> {:?}",t));
        }

//        node_should_select.event.display_graphviz();

        let selected_nodes2 = selected_nodes.clone_ref();

        frp! {
            mouse_down_position    = mouse.position.sample        (&mouse.on_down);
            selection_zero         = source::<Position>           ();
            selection_size_down    = mouse.position.map2          (&mouse_down_position,|m,n|{m-n});
            selection_size_if_down = selection_size_down.gate     (&mouse.is_down);
            selection_size_on_down = selection_zero.sample        (&mouse.on_down);
            selection_size         = selection_size_if_down.merge (&selection_size_on_down);

            mouse_down_target      = mouse.on_down.map            (enclose!((scene) move |_| scene.mouse.target.get()));


            add_node_with_cursor_pos = events.add_node_under_cursor.map2(&mouse.position, |_,pos| { *pos });

            add_node_unified = events.add_node_at.merge(&add_node_with_cursor_pos);

            _node_added = add_node_unified.map(enclose!((node_set,on_node_press,display_object) move |pos| {
                let node = Node::new();
                let weak_node = node.downgrade();
                node.view.events.mouse_down.map("foo",enclose!((on_node_press) move |_| {
                    on_node_press.event.emit(Some(weak_node.clone_ref()))
                }));

                display_object.add_child(&node);
                node.mod_position(|t| {
                    t.x += pos.x as f32;
                    t.y += pos.y as f32;
                });

                node_set.insert(node);

            }));

            _graph_cleared = events.clear_graph.map(enclose!((node_set) move |()| {
                node_set.clear();
            }));

            _bar = events.remove_selected_nodes.map(enclose!((selected_nodes2) move |_| {
                selected_nodes2.for_each_taken(|node| {
                    node_set.remove(&node);
                })
            }));

            _baz = node_select.map(move |opt_node| {
                opt_node.for_each_ref(|weak_node| {
                    weak_node.upgrade().map(|node| {
                        selected_nodes2.deselect_all();
                        node.events.select.event.emit(());
                        selected_nodes2.insert(&node);
                    })
                })
            })
        }


        mouse.on_down.map("cursor_press", enclose!((cursor) move |_| {
            cursor.events.press.event.emit(());
        }));

        mouse.on_up.map("cursor_release", enclose!((cursor) move |_| {
            cursor.events.release.event.emit(());
        }));

        mouse.position.map("cursor_position", enclose!((cursor) move |p| {
            cursor.set_position(Vector2::new(p.x as f32,p.y as f32));
        }));

        selection_size.map("cursor_size", enclose!((cursor) move |p| {
            cursor.set_selection_size(Vector2::new(p.x as f32,p.y as f32));
        }));

        let on_bg_press2    = on_bg_press.clone_ref();
        mouse_down_target.map("mouse_down_target", enclose!((scene) move |target| {
            match target {
                display::scene::Target::Background => {
                    on_bg_press2.event.emit(());
//                    selected_nodes2.deselect_all();
                }
                display::scene::Target::Symbol {instance_id,..} => {
                    scene.shapes.get_mouse_target(&(*instance_id as usize)).for_each(|target| {
                        target.mouse_down().for_each(|t| t.event.emit(()));
                    })
                }
            }
        }));


//        frp! { mouse_down_position    = mouse.position.sample (&mouse.on_down)    }
//        frp! { mouse_position_if_down = mouse.position.gate   (&mouse.is_down) }
//
//        let final_position_ref_event  = frp::Recursive::<frp::EventData<Position>>::new_named("final_position_ref");
//        let final_position_ref        = frp::Dynamic::from(&final_position_ref_event);
//
//        frp! { pos_diff_on_down = mouse_down_position.map2    (&final_position_ref,|m,f|{m-f}) }
//        frp! { final_position   = mouse_position_if_down.map2 (&pos_diff_on_down  ,|m,f|{m-f}) }
//
//        final_position_ref_event.initialize(&final_position);
//
//        final_position_ref.event.set_display_id(final_position.event.display_id());
//        final_position_ref.behavior.set_display_id(final_position.event.display_id());
//
//        frp! {
//            foo = final_position.map(|p| {
//                println!("POS: {:?}",p);
//            })
//        }




        let add_node_ref = events.add_node_under_cursor.clone_ref();
        let remove_selected_nodes_ref = events.remove_selected_nodes.clone_ref();
        let selected_nodes2 = selected_nodes.clone_ref();
        let world2 = world.clone_ref();
        let c: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |val| {
            let val = val.unchecked_into::<web_sys::KeyboardEvent>();
            let key = val.key();
            if      key == "n"         { add_node_ref.event.emit(()) }
            else if key == "Backspace" {
                remove_selected_nodes_ref.event.emit(())
            }
            else if key == "p" {
                selected_nodes2.for_each_taken(|node| {
                    world2.scene().remove_child(&node);
                })
            }
        }));
        web::document().add_event_listener_with_callback("keydown",c.as_ref().unchecked_ref()).unwrap();
        c.forget();


        Self {events,selected_nodes,display_object}
    }
}

impl<'a> From<&'a GraphEditor> for &'a display::object::Node {
    fn from(graph_editor: &'a GraphEditor) -> Self {
        &graph_editor.display_object
    }
}
