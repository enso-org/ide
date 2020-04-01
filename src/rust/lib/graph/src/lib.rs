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
    pub add_node              : frp::Dynamic<()>,
    pub remove_selected_nodes : frp::Dynamic<()>,
}

impl Events {
    pub fn new() -> Self {
        frp! {
            add_node              = source::<()> ();
            remove_selected_nodes = source::<()> ();
        }
        Self {add_node,remove_selected_nodes}
    }
}

#[derive(Debug)]
pub struct GraphEditor {
    pub events         : Events,
    pub selected_nodes : WeakNodeSelectionSet,
}

impl GraphEditor {
    pub fn new(world: &World) -> Self {
        let scene  = world.scene();
        let cursor = Cursor::new();
        web::body().set_style_or_panic("cursor","none");


        world.add_child(&cursor);


        let events = Events::new();
    //    web::body().set_style_or_panic("cursor","none");

        let mouse = &scene.mouse.frp;

        let node_set = NodeSet::default();

        let selected_nodes = WeakNodeSelectionSet::default();

        let selected_nodes2 = selected_nodes.clone_ref();

        frp! {
            mouse_down_position    = mouse.position.sample        (&mouse.on_down);
            selection_zero         = source::<Position>           ();
            selection_size_down    = mouse.position.map2          (&mouse_down_position,|m,n|{m-n});
            selection_size_if_down = selection_size_down.gate     (&mouse.is_down);
            selection_size_on_down = selection_zero.sample        (&mouse.on_down);
            selection_size         = selection_size_if_down.merge (&selection_size_on_down);


            mouse_down_target      = mouse.on_down.map            (enclose!((scene) move |_| scene.mouse.target.get()));


            node_mouse_down = source::<Option<WeakNode>> ();

            _foo = events.add_node.map2(&mouse.position, enclose!((node_set,node_mouse_down,world) move |_,pos| {
                let node = Node::new();
                let weak_node = node.downgrade();
                node.view.events.mouse_down.map("foo",enclose!((node_mouse_down) move |_| {
                    node_mouse_down.event.emit(Some(weak_node.clone_ref()))
                }));

                world.add_child(&node);
                node.mod_position(|t| {
                    t.x += pos.x as f32;
                    t.y += pos.y as f32;
                });

                node_set.insert(node);

            }));

            _bar = events.remove_selected_nodes.map(enclose!((selected_nodes2) move |_| {
                selected_nodes2.for_each_taken(|node| {
                    node_set.remove(&node);
                })
            }));

            _baz = node_mouse_down.map(move |opt_node| {
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

        let selected_nodes2 = selected_nodes.clone_ref();
        mouse_down_target.map("mouse_down_target", enclose!((scene) move |target| {
            match target {
                display::scene::Target::Background => {
                    selected_nodes2.deselect_all();
                }
                display::scene::Target::Symbol {symbol_id:_, instance_id} => {
                    scene.shapes.get_mouse_target(&(*instance_id as usize)).for_each(|target| {
                        target.mouse_down().for_each(|t| t.event.emit(()));
                    })
                }
            }
        }));

        let add_node_ref = events.add_node.clone_ref();
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


        Self {events,selected_nodes}
    }
}
