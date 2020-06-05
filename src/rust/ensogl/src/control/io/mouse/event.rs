//! This module defines possible mouse events.

use crate::prelude::*;

use crate::control::io::mouse::button::*;
use crate::system::web::dom::Shape;



// =============
// === Event ===
// =============

macro_rules! define_events {
    ( $( $js_event:ident :: $name:ident ),* $(,)? ) => {$(
        /// Mouse event wrapper.
        #[derive(Debug,Clone,From,Shrinkwrap)]
        pub struct $name {
            #[shrinkwrap(main_field)]
            raw   : web_sys::$js_event,
            shape : Shape,
        }
        impl $name {

            /// Constructor.
            pub fn new(raw:web_sys::$js_event,shape:Shape) -> Self {
                Self {raw,shape}
            }

            /// The Y coordinate of the mouse pointer relative to the position of the padding edge
            /// of the target node.
            pub fn offset_y(&self) -> i32 {
                self.shape.height as i32 - self.raw.offset_y()
            }

            /// The Y coordinate of the mouse pointer in local (DOM content) coordinates.
            pub fn client_y(&self) -> i32 {
                self.shape.height as i32 - self.raw.client_y()
            }

            /// The Y coordinate of the mouse pointer in global (screen) coordinates.
            pub fn screen_y(&self) -> i32 {
                self.shape.height as i32 - self.raw.screen_y()
            }

            /// Translation of the button property to Rust `Button` enum.
            pub fn button(&self) -> Button {
                Button::from_code(self.raw.button())
            }
        }
    )*};
}

define_events! {
    MouseEvent::OnDown,
    MouseEvent::OnUp,
    MouseEvent::OnMove,
    MouseEvent::OnLeave,
    WheelEvent::OnWheel,
}
