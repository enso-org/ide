//! Utilities for DOM events.

pub mod listener;

use crate::prelude::*;

use js_sys::Function;
use web_sys::EventTarget;



// =============
// === Event ===
// =============

/// This trait represents a kind of event that may fire from some specific JS object.
///
/// For example, `WebSocket.close` is such an event.
pub trait Event {
    /// The type of the event -- it will be the type of value passed to the event listeners.
    /// For example `web_sys::CloseEvent`.
    type Interface : AsRef<web_sys::Event>;

    /// The type of the object emitting event, e.g. `web_sys::WebSocket`.
    type Target : AsRef<EventTarget> + AsRef<JsValue> + Clone + PartialEq;

    /// The name of the event. For example `"close"`.
    const NAME:&'static str;

    /// Adds a given function to the event's target as an event listener. It will be called each
    /// time event occurs until listener is removed through `remove_listener`.
    fn add_listener(target:&Self::Target, listener:&Function) {
        EventTarget::add_event_listener_with_callback(target.as_ref(), Self::NAME, listener).unwrap()
    }

    /// Remove the event listener. The `add_listener` method should have been called before with
    /// the very same function argument.
    fn remove_listener(target:&Self::Target, listener:&Function) {
        EventTarget::remove_event_listener_with_callback(target.as_ref(), Self::NAME, listener).unwrap()
    }
}
