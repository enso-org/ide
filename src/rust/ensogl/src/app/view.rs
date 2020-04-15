
use crate::prelude::*;

use crate::display::world::World;
use crate::frp::io::keyboard;
use crate::frp;
use super::command;
use super::shortcut;






// ============
// === View ===
// ============

pub trait View : command::Provider {
    fn new(world:&World) -> Self;
}



// ==================
// === Definition ===
// ==================

#[derive(Debug)]
pub struct Definition {
    shortcut_handles : Vec<shortcut::Handle>
}



// ================
// === Registry ===
// ================

#[derive(Debug,Clone,CloneRef)]
pub struct Registry {
    pub logger            : Logger,
    pub display           : World,
    pub command_registry  : command::Registry,
    pub shortcut_registry : shortcut::Registry,
    pub definitions       : Rc<RefCell<HashMap<String,Definition>>>,
}

impl Registry {
    pub fn create
    ( logger            : &Logger
    , display           : &World
    , command_registry  : &command::Registry
    , shortcut_registry : &shortcut::Registry
    ) -> Self {
        let logger            = logger.sub("view_registry");
        let display           = display.clone_ref();
        let command_registry  = command_registry.clone_ref();
        let shortcut_registry = shortcut_registry.clone_ref();
        let definitions       = default();
        Self {logger,display,command_registry,shortcut_registry,definitions}
    }

    pub fn register<V:View>(&self) {
        let label            = V::view_name().into();
        let shortcut_handles = V::default_shortcuts().into_iter().map(|shortcut| {
            self.shortcut_registry.add(shortcut)
        }).collect();
        let definition = Definition {shortcut_handles};
        self.definitions.borrow_mut().insert(label,definition);
        self.command_registry.register::<V>();
    }

    pub fn new<V:View>(&self) -> V {
        let label          = V::view_name();
        let was_registered = self.definitions.borrow().get(label).is_some();
        if !was_registered {
            warning!(&self.logger,
                "The view '{label}' was created but never registered. You should always \
                register available views as soon as possible to enable their default shortcuts and \
                spread the information about their API.");
            self.register::<V>();
        }
        let view = V::new(&self.display);
        self.command_registry.register_instance(&view);
        view
    }
}
