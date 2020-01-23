//! This module defines mouse event bindings.

use crate::prelude::*;

use crate::control::io::mouse::button::*;


// =============
// === Event ===
// =============

/// Abstraction for every event type.
pub trait Event {
    /// Accessor to the underlying `MouseEvent` in the JavaScript world.
    fn raw(&self) -> &web_sys::MouseEvent;

    /// Translation of the button property to Rust `Button` enum.
    fn button(&self) -> Button {
        Button::from_code(self.raw().button())
    }
}


// === Events ===

macro_rules! define_events {
    ( $( $name:ident ),* $(,)? ) => {$(
        #[derive(Debug,Clone,From,Shrinkwrap)]
        pub struct $name {
            raw: web_sys::MouseEvent
        }
        impl Event for $name {
            fn raw(&self) -> &web_sys::MouseEvent {
                &self.raw
            }
        }
    )*};
}

define_events!(OnDown,OnUp,OnMove);
