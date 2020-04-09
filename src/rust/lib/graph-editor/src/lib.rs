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

#![recursion_limit="256"]

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
use enso_frp::Position;
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

    pub fn for_each<F:Fn(Node)>(&self,f:F) {
        self.data.borrow().iter().for_each(|(_,node)| { node.upgrade().for_each(|n| f(n)) })
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
        self.for_each_taken(|node| node.events.deselect.emit(()));
    }
}

#[derive(Debug)]
pub struct NodesEvents {
    pub press              : frp::Source<Option<WeakNode>>,
    pub select             : frp::Stream<Option<WeakNode>>,
    pub translate_selected : frp::Source<Position>,
}


#[derive(Debug)]
pub struct Events {
    pub network                  : frp::Network,
    pub add_node_under_cursor    : frp::Source,
    pub add_node_at              : frp::Source<Position>,
    pub remove_selected_nodes    : frp::Source,
    pub clear_graph              : frp::Source,
    pub nodes                    : NodesEvents,
}

impl Events {
    pub fn new(network:frp::Network, nodes:NodesEvents) -> Self {
        frp::extend_network! { network
            def add_node_under_cursor    = source::<()>       ();
            def add_node_at              = source::<Position> ();
            def remove_selected_nodes    = source::<()>       ();
            def clear_graph              = source::<()>       ();
        }
        Self {network,add_node_under_cursor,add_node_at,remove_selected_nodes,clear_graph,nodes}
    }
}

#[derive(Debug)]
pub struct GraphEditor {
    pub frp            : Events,
    pub selected_nodes : WeakNodeSelectionSet,
    pub display_object : display::object::Node,
    pub node_set       : NodeSet,
}

pub struct SelectionNetwork<T:frp::Data> {
    pub press      : frp::Source<T>,
    pub is_pressed : frp::Stream<bool>,
    pub mouse_pos_on_press : frp::Stream<Position>,
    pub select     : frp::Stream<T>
}

impl<T:frp::Data> SelectionNetwork<T> {
    pub fn new(selection_target:&frp::Network,mouse:&frp::io::Mouse) -> Self {
        frp::extend_network! { selection_target
            def press          = source::<T> ();
            def press_bool     = press.map(|_| true);
            def release_bool   = mouse.release.map(|_| false);
            def is_pressed     = press_bool.merge(&release_bool);
            def was_pressed    = is_pressed.previous();
            def mouse_release  = mouse.release.gate(&was_pressed);
            def mouse_pos_on_press   = mouse.position.sample(&press);
            def pos_on_release = mouse.position.sample(&mouse_release);
            def should_select  = pos_on_release.map3(&mouse_pos_on_press,&mouse.distance,Self::check);
            def release        = press.sample(&mouse_release);
            def select         = release.gate(&should_select);
        }
        Self {press,is_pressed,mouse_pos_on_press,select}
    }

    fn check(end:&Position, start:&Position, diff:&f32) -> bool {
        (end-start).length() <= diff * 2.0
    }
}

impl GraphEditor {

    pub fn add_node(&self) -> WeakNode {
        let node      = Node::new();
        let weak_node = node.downgrade();
        let network   = &self.frp.network;
        let on_node_press = self.frp.nodes.press.clone_ref();
        frp::new_subnetwork! { [network,node.view.events.network]
            def foo_ = node.view.events.mouse_down.map(move |_|
                on_node_press.emit(Some(weak_node.clone_ref()))
            );
        }
        self.display_object.add_child(&node);
        let weak_node = node.downgrade();
        self.node_set.insert(node);
        weak_node
    }

    pub fn new(world: &World) -> Self {
        let scene  = world.scene();
        let cursor = Cursor::new();
        web::body().set_style_or_panic("cursor","none");
        world.add_child(&cursor);

        let display_object = display::object::Node::new(Logger::new("GraphEditor"));

        let mouse = &scene.mouse.frp;

        let network   = frp::Network::new();

        let nodes_frp = SelectionNetwork::<Option<WeakNode>>::new(&network,mouse);
        let bg_frp    = SelectionNetwork::<()>::new(&network,mouse);


        let on_node_press = nodes_frp.press.clone_ref(); // FIXME
        let node_select = nodes_frp.select.clone_ref(); // FIXME

        let on_bg_press = bg_frp.press.clone_ref(); // FIXME
        let bg_select   = bg_frp.select.clone_ref(); // FIXME

        frp::extend_network! { network
            def translate_selected_nodes = source::<Position>();
        }

        let nodes_events = NodesEvents {press:on_node_press.clone(), select:node_select.clone_ref(),translate_selected:translate_selected_nodes.clone_ref()};



        let events = Events::new(network,nodes_events);
    //    web::body().set_style_or_panic("cursor","none");


        let node_set = NodeSet::default();

        let selected_nodes = WeakNodeSelectionSet::default();

        let selected_nodes2 = selected_nodes.clone_ref();

        let network = &events.network;




        frp::extend_network! { network
            def _bg_selection = bg_select.map(move |_| {
                selected_nodes2.deselect_all();
            });
        }

        let translate_selected_nodes2 = translate_selected_nodes.clone_ref();
        frp::extend_network! { network
            let target      = nodes_frp.press.clone_ref(); // FIXME
            let is_pressed  = nodes_frp.is_pressed.clone_ref(); // FIXME
            def translation = mouse.translation.gate(&is_pressed);
            def _move_node  = translation.map2(&target,|t,opt_node| {
                opt_node.for_each_ref(|weak_node| {
                    weak_node.upgrade().for_each(|node| {
                        node.mod_position(|p| {
                            p.x += t.x;
                            p.y += t.y;
                        })
                    })
                })
            });


            let selected_nodes2 = selected_nodes.clone_ref();

            def _move_node = translate_selected_nodes.map(move |t| {
                selected_nodes2.for_each(|node| {
                    node.mod_position(|p| {
                            p.x += t.x;
                            p.y += t.y;
                        })
                })
            });

        }


//        node_should_select.event.display_graphviz();

        let selected_nodes2 = selected_nodes.clone_ref();

        frp::extend_network! { network
            let is_bg_pressed  = bg_frp.is_pressed.clone_ref(); // FIXME

            trace is_bg_pressed;

            def mouse_down_position    = mouse.position.sample        (&mouse.press);
            def selection_zero         = source::<Position>           ();
            def selection_size_down    = mouse.position.map2          (&mouse_down_position,|m,n|{m-n});
            def selection_size_if_down = selection_size_down.gate     (&is_bg_pressed);
            def selection_size_on_down = selection_zero.sample        (&mouse.press);


            def selection_size         = selection_size_if_down.merge (&selection_size_on_down);

            def mouse_down_target      = mouse.press.map            (enclose!((scene) move |_| scene.mouse.target.get()));


            def add_node_with_cursor_pos = events.add_node_under_cursor.map2(&mouse.position, |_,pos| { *pos });

            def add_node_unified = events.add_node_at.merge(&add_node_with_cursor_pos);

            def _node_added = add_node_unified.map(enclose!((network,node_set,on_node_press,display_object) move |pos| { // on_node_press
                let node = Node::new();
                let weak_node = node.downgrade();
                frp::new_subnetwork! { [network,node.view.events.network]
                    def foo_ = node.view.events.mouse_down.map(enclose!((on_node_press) move |_| {
                        on_node_press.emit(Some(weak_node.clone_ref()))
                    }));
                }
                display_object.add_child(&node);
                node.mod_position(|t| {
                    t.x += pos.x as f32;
                    t.y += pos.y as f32;
                });

                node_set.insert(node);

            }));

            def _graph_cleared = events.clear_graph.map(enclose!((node_set) move |()| {
                node_set.clear();
            }));

            def _bar = events.remove_selected_nodes.map(enclose!((node_set,selected_nodes2) move |_| {
                selected_nodes2.for_each_taken(|node| {
                    node_set.remove(&node);
                })
            }));

            def _baz = node_select.map(move |opt_node| {
                opt_node.for_each_ref(|weak_node| {
                    weak_node.upgrade().map(|node| {
                        selected_nodes2.deselect_all();
                        node.events.select.emit(());
                        selected_nodes2.insert(&node);
                    })
                })
            });
        }


        frp::extend_network! { network

            def _cursor_press = mouse.press.map(enclose!((cursor) move |_| {
                cursor.events.press.emit(());
            }));

            def _cursor_release = mouse.release.map(enclose!((cursor) move |_| {
                cursor.events.release.emit(());
            }));

            def _cursor_position = mouse.position.map(enclose!((cursor) move |p| {
                cursor.set_position(Vector2::new(p.x as f32,p.y as f32));
            }));

            def _cursor_size = selection_size.map(enclose!((cursor) move |p| {
                cursor.set_selection_size(Vector2::new(p.x as f32,p.y as f32));
            }));

            let on_bg_press2    = on_bg_press.clone_ref();
            def _mouse_down_target = mouse_down_target.map(enclose!((scene) move |target| {
                match target {
                    display::scene::Target::Background => {
                        on_bg_press2.emit(());
    //                    selected_nodes2.deselect_all();
                    }
                    display::scene::Target::Symbol {instance_id,..} => {
                        scene.shapes.get_mouse_target(&(*instance_id as usize)).for_each(|target| {
                            target.mouse_down().for_each(|t| t.emit(()));
                        })
                    }
                }
            }));

        }


        let add_node_ref = events.add_node_under_cursor.clone_ref();
        let remove_selected_nodes_ref = events.remove_selected_nodes.clone_ref();
        let selected_nodes2 = selected_nodes.clone_ref();
        let world2 = world.clone_ref();
        let c: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |val| {
            let val = val.unchecked_into::<web_sys::KeyboardEvent>();
            let key = val.key();
            if      key == "n"         { add_node_ref.emit(()) }
            else if key == "Backspace" {
                remove_selected_nodes_ref.emit(())
            }
            else if key == "p" {
                selected_nodes2.for_each_taken(|node| {
                    world2.scene().remove_child(&node);
                })
            }
        }));
        web::document().add_event_listener_with_callback("keydown",c.as_ref().unchecked_ref()).unwrap();
        c.forget();


        Self {frp:events,selected_nodes,display_object,node_set}
    }
}

impl<'a> From<&'a GraphEditor> for &'a display::object::Node {
    fn from(graph_editor: &'a GraphEditor) -> Self {
        &graph_editor.display_object
    }
}
