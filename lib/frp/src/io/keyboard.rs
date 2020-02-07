use crate::prelude::*;

use crate::nodes::*;
use crate::frp_def;

// ================
// === Keyboard ===
// ================

pub enum Navigation {
    Left,Right,Top,Bottom,Home,End,PageUp,PageDown
}

pub enum Modifier {
    Control,Alt,Shift
}

pub enum Key {
    Tab,Enter,Backspace,Delete,
    AlphaNum(char),
    Navigation(Navigation),
    Modifier(Modifier)
}

/// Keyboard FRP bindings.
#[derive(Debug)]
pub struct Keyboard {
    /// The mouse up event.
    pub key_pressed: Dynamic<Key>,
    /// The mouse down event.
    pub key_released: Dynamic<Key>,
    pub is_alt_pressed: Dynamic<bool>,
    pub is_control_pressed: Dynamic<bool>,
    pub is_shift_pressed: Dynamic<bool>,
}

impl Default for Keyboard {
    fn default() -> Self {
        frp_def! { keyboard.key_pressed       = source() }
        frp_def! { keyboard.key_released      = source() }
        frp_def! { mouse.position  = source() }
        frp_def! { mouse.down_bool = down.constant(true) }
        frp_def! { mouse.up_bool   = up.constant(false) }
        frp_def! { mouse.is_down   = down_bool.merge(&up_bool) }
        Self {up,down,is_down,position}
    }
}

impl Mouse {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }
}
