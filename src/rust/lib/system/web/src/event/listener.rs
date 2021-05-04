use crate::prelude::*;

use crate::closure::storage::ClosureFn;
use crate::closure::storage::OptionalFmMutClosure;



// ============
// === Slot ===
// ============

/// A single event listener slot.
///
/// Stores a closure that can be registered as an event listener.
///
/// Slot will register the closure as event listener whenever both closure and target are provided.
///
/// Caveat: this listener holds a reference to the target while it is registered.
/// Be sure not to leak this value nor have it dependent on target destruction.
#[derive(Derivative)]
#[derivative(Debug(bound="Event::Interface: Debug"))]
pub struct Slot<Event:crate::event::Event> {
    logger     : Logger,
    #[derivative(Debug="ignore")]
    target     : Option<Event::Target>,
    js_closure : OptionalFmMutClosure<Event::Interface>,
}

impl<Event:crate::event::Event> Slot<Event> {
    /// Create a new `Slot`. As the initial target is provided, the listener will register once it
    /// gets a callback (see [[set_callback]]).
    pub fn new(target:&Event::Target, logger:impl AnyLogger) -> Self {
        Self {
            logger     : Logger::sub(logger,Event::NAME),
            target     : Some(target.clone()),
            js_closure : default(),
        }
    }

    /// Register the event listener if both target and callback are set.
    fn add_if_active(&mut self) {
        if let (Some(target), Some(function)) = (self.target.as_ref(), self.js_closure.js_ref()) {
            debug!(self.logger,"Attaching the callback.");
            Event::add_listener(target,function)
        }
    }

    /// Unregister the event listener if both target and callback are set.
    fn remove_if_active(&mut self) {
        if let (Some(target), Some(function)) = (self.target.as_ref(), self.js_closure.js_ref()) {
            debug!(self.logger,"Detaching the callback.");
            Event::remove_listener(target, function)
        }
    }

    /// Move this event listener to a different target.
    pub fn set_target(&mut self, target:&Event::Target) {
        // Prevent spurious reattaching that could affect listeners order.
        if Some(target) != self.target.as_ref() {
            self.remove_if_active();
            self.target = Some(target.clone());
            self.add_if_active()
        }
    }

    /// Assign a new event callback closure and register it in the target.
    ///
    /// If the listener was registered with the previous closure, it will unregister first.
    ///
    /// Caveat: using this method will move the event listener to the end of the registered
    /// callbacks. This will affect the order of callback calls.
    pub fn set_callback(&mut self, f:impl ClosureFn<Event::Interface>) {
        self.remove_if_active();
        self.js_closure.wrap(f);
        self.add_if_active()
    }

    /// Erase the callback.
    ///
    /// The stored closure will be dropped and event listener unregistered.
    pub fn clear_callback(&mut self) {
        self.remove_if_active();
        self.js_closure.clear();
    }

    /// Detach and attach the listener to the target.
    ///
    /// The purpose is to move this slot to the end of the listeners list.
    pub fn reattach(&mut self) {
        self.remove_if_active();
        self.add_if_active();
    }
}

/// Unregister listener on drop.
impl<Event:crate::event::Event> Drop for Slot<Event> {
    fn drop(&mut self) {
        self.remove_if_active();
    }
}
