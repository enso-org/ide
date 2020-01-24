//! This module defines mouse event bindings.

use crate::prelude::*;

use crate::control::io::mouse::button::*;


// =============
// === Event ===
// =============

macro_rules! define_events {
    ( $( $name:ident ),* $(,)? ) => {$(
        #[derive(Debug,Clone,From,Shrinkwrap)]
        pub struct $name {
            raw: web_sys::MouseEvent
        }
        impl $name {
            /// Translation of the button property to Rust `Button` enum.
            pub fn button(&self) -> Button {
                Button::from_code(self.raw.button())
            }
        }
    )*};
}

define_events!(OnDown,OnUp,OnMove);
