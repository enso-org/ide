use bigint::uint::U256;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::keyboard_engine::callback_registry::*;

type BindingsMap = HashMap<U256, CallbackRegistry>;

#[derive(Default)]
pub struct Bindings {
    pub data: RefCell<BindingsMap>,
}

impl Bindings {
    pub fn new() -> Self{
        Self {
            data: RefCell::new(HashMap::new())
        }
    }

    pub fn add<F: CallbackMut>
    (&self, key: U256, callback: F) -> CallbackHandle {
        let mut data = self.data.borrow_mut();
        match data.get_mut(&key) {
            Some(registry) => {
                registry.add(callback)
            }
            None => {
                let mut registry = CallbackRegistry::new();
                let handle = registry.add(callback);
                data.insert(key, registry);
                handle
            }
        }
    }

    pub fn remove(&self, key: U256) {
        self.data.borrow_mut().remove(&key);
    }

    pub fn call_by_key(&self, key: U256) {
        if let Some(registry) = self.data.borrow_mut().get_mut(&key) {
            registry.call();
        }
    }
}
