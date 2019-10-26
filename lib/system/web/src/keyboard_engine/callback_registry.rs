use std::rc::{Rc, Weak};

pub trait CallbackMut = FnMut() + 'static;

pub struct Guard {
    weak: Weak<()>
}

impl Guard {
    pub fn exists(&self) -> bool {
        self.weak.upgrade().is_some()
    }
}

#[derive(Default)]
pub struct CallbackHandle {
    rc: Rc<()>
}

impl CallbackHandle {
    pub fn guard(&self) -> Guard {
        let weak = Rc::downgrade(&self.rc);
        Guard {weak}
    }
}

pub struct CallbackRegistry {
    pub registry: Vec<(Guard, Box<dyn CallbackMut>)>
}

impl CallbackRegistry {
    pub fn new() -> Self {
        Self {
            registry: Vec::new()
        }
    }

    pub fn call(&mut self) {
        self.drop_orphaned_callbacks();
        self.registry
            .iter_mut()
            .for_each(|(_, func)| func());
    }

    pub fn add<F: CallbackMut>
    (&mut self, callback: F) -> CallbackHandle {
        let handle = CallbackHandle::default();
        let guard = handle.guard();
        self.registry.push( (guard, Box::new(callback)) );
        handle
    }

    pub fn drop_orphaned_callbacks(&mut self) {
        self.registry.retain(|(guard, _)| guard.exists());
    }
}
