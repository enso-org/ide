//! Root module for Input / Output FRP bindings

pub mod keyboard;
pub mod keyboard_old;
pub mod mouse;

pub use keyboard_old::Keyboard;
pub use mouse::Mouse;
