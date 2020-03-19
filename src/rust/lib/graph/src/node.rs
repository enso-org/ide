
use crate::prelude::*;

use crate::{HasSprite, ChangeType};

use ensogl::control::callback::CallbackMut1;
use ensogl::data::color::Srgba;
use ensogl::display;
use ensogl::display::Sprite;
use ensogl::math::Vector2;
use ensogl::math::Vector3;
use logger::Logger;
use std::any::TypeId;
use enso_prelude::std_reexports::fmt::{Formatter, Error};

type EditCallback = Box<dyn Fn(&Node) + 'static>;

#[derive(Default)]
pub struct OnEditCallbacks {
    pub label_changed    : Option<EditCallback>,
    pub color_changed    : Option<EditCallback>,
    pub position_changed : Option<EditCallback>,
    pub removed          : Option<EditCallback>,
}

impl Debug for OnEditCallbacks {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("node::OnEditCallback")
    }
}

#[derive(Debug,Default)]
pub struct NodeData {
    label        : String,
    color        : Srgba,
}

#[derive(Debug,Clone)]
pub struct Node {
    logger         : Logger,
    sprite         : Rc<CloneCell<Option<Sprite>>>,
    display_object : display::object::Node,
    data           : Rc<RefCell<NodeData>>,
    callbacks      : Rc<RefCell<OnEditCallbacks>>,
}

impl Node {
    pub fn new() -> Self {
        let logger = Logger::new("node");
        let sprite : Rc<CloneCell<Option<Sprite>>>        = default();
        let display_object = display::object::Node::new(&logger);
        display_object.set_on_show_with(enclose!((sprite,display_object) move |this,scene| {
            let type_id      = TypeId::of::<Node>();
            let shape_system = scene.lookup_shape(&type_id).unwrap();
            let new_sprite   = shape_system.new_instance();
            this.add_child_tmp(&display_object,&new_sprite);
            new_sprite.size().set(Vector2::new(200.0,200.0));
            sprite.set(Some(new_sprite));
        }));

        display_object.set_on_hide_with(|scene| {
            println!("set_on_hide_with");
        });
        let data      = default();
        let callbacks = default();
        Self {logger,sprite,display_object,data,callbacks}
    }

    pub fn set_on_edit_callbacks(&self, callbacks: OnEditCallbacks) {
        *self.callbacks.borrow_mut() = callbacks
    }
}


impl Node {
    pub fn set_position(&self, pos:Vector3<f32>, change_type:ChangeType) {
        self.display_object.set_position(pos);
        if let ChangeType::FromGUI = change_type {
            self.call_edit_callback(&self.callbacks.borrow().position_changed);
        }
    }

    pub fn set_label(&self, new_label:String, change_type:ChangeType) {
        self.data.borrow_mut().label = new_label;
        //TODO[ao] update sprites
        if let ChangeType::FromGUI = change_type {
            self.call_edit_callback(&self.callbacks.borrow().label_changed);
        }
    }

    pub fn set_color(&self, new_color:Srgba, change_type:ChangeType) {
        self.data.borrow_mut().color = new_color;
        //TODO[ao] update sprites
        if let ChangeType::FromGUI = change_type {
            self.call_edit_callback(&self.callbacks.borrow().color_changed);
        }
    }

    pub fn remove_from_graph(&self) {
        todo!()
    }

    fn call_edit_callback(&self, callback:&Option<EditCallback>) {
        if let Some(callback) = callback {
            callback(self)
        }
    }
}

impl HasSprite for Node {
    fn set_sprite(&self, sprite:&Sprite) {
        self.sprite.set(Some(sprite.clone()))
    }
}

impl<'t> From<&'t Node> for &'t display::object::Node {
    fn from(node:&'t Node) -> Self {
        &node.display_object
    }
}