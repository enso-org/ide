use crate::prelude::*;

use crate::closure::storage::ClosureFn;
use crate::closure::storage::OptionalFmMutClosure;
use crate::event::Event as JsEvent;

use js_sys::Function;

#[derive(Derivative)]
#[derivative(Debug(bound="Event::Interface: Debug"))]
pub struct ListenerSlot<Event:JsEvent> {
    logger     : Logger,
    #[derivative(Debug="ignore")]
    target     : Option<Event::Target>,
    js_closure : OptionalFmMutClosure<Event::Interface>,
}

impl<Event:JsEvent> Default for ListenerSlot<Event> {
    fn default() -> Self {
        Self {
            logger:Logger::new(Event::NAME),
            target:default(),
            js_closure:default(),
        }
    }
}

impl<Event:JsEvent> ListenerSlot<Event> {
    pub fn new(target:&Event::Target, logger:impl AnyLogger) -> Self {
        Self {
            logger     : Logger::sub(logger,Event::NAME),
            target     : Some(target.clone()),
            js_closure : default(),
        }
    }

    pub fn target(&self) -> Result<&Event::Target,JsValue> {
        let err = || js_sys::Error::new("No target object provided.");
        self.target.as_ref().ok_or_else(err).map_err(Into::into)
    }

    pub fn js_function(&self) -> Result<&Function,JsValue> {
        let err = || js_sys::Error::new("No closure has been set.");
        self.js_closure.js_ref().ok_or_else(err).map_err(Into::into)
    }

    fn add_if_active(&mut self) {
        if let (Ok(target),Ok(function)) = (self.target(), self.js_function()) {
            info!(self.logger,"Attaching the callback.");
            Event::add_listener(target,function)
        }
    }

    fn remove_if_active(&mut self) {
        if let (Ok(target),Ok(function)) = (self.target(), self.js_function()) {
            info!(self.logger,"Detaching the callback.");
            Event::remove_listener(target, function)
        }
    }

    pub fn set_target(&mut self, target:&Event::Target) {
        self.remove_if_active();
        self.target = Some(target.clone());
        self.add_if_active()
    }

    pub fn set_callback(&mut self, f:impl ClosureFn<Event::Interface>) {
        self.remove_if_active();
        self.js_closure.wrap(f);
        self.add_if_active()
    }

    pub fn clear_callback(&mut self) {
        self.remove_if_active();
        self.js_closure.clear();
    }
}
