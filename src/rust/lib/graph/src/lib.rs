#![feature(bool_to_option)]
#![feature(drain_filter)]
#![feature(trait_alias)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod node;

use prelude::*;

use ensogl::display;
use ensogl::display::Sprite;
use logger::Logger;
use enso_prelude::std_reexports::fmt::{Formatter, Error};

pub mod prelude {
    pub use enso_prelude::*;
}


// =========================
// === Library Utilities ===
// =========================

pub trait HasSprite {
    fn set_sprite(&self, sprite:&Sprite);
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum ChangeType {
    FromGUI, FromController
}



// =============
// === Graph ===
// =============

#[derive(Default)]
struct OnEditCallbacks {
    node_added : Option<Box<dyn Fn(&node::Node) + 'static>>,
}

impl Debug for OnEditCallbacks {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("graph::OnEditCallbacks")
    }
}

#[derive(Debug,Default)]
struct GraphData {
    nodes : Vec<node::Node>,
}

#[derive(Debug)]
struct Graph {
    data           : Rc<RefCell<GraphData>>,
    display_object : display::object::Node,
    callbacks      : Rc<RefCell<OnEditCallbacks>>,
    logger         : Logger,
}

impl Graph {
    pub fn new() -> Self {
        let logger         = Logger::new("graph");
        let data           = default();
        let display_object = display::object::Node::new(&logger);
        let callbacks      = default();
        Self {data,display_object,callbacks,logger}
    }

    pub fn set_on_edit_callbacks(&self, callbacks: OnEditCallbacks) {
        *self.callbacks.borrow_mut() = callbacks
    }
}

impl Graph {
    pub fn add_node(&self, new_node:node::Node, change_type:ChangeType) {
        self.display_object.add_child(&new_node);
        self.data.borrow_mut().nodes.push(new_node.clone());
        if let ChangeType::FromGUI = change_type {
            if let Some(callback) = &self.callbacks.borrow().node_added {
                callback(&new_node)
            }
        }
    }
}
