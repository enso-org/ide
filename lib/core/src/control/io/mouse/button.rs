//! This module defines bindings to mouse buttons.

use crate::prelude::*;


// ===================
// === MouseButton ===
// ===================

/// An enumeration representing the mouse buttons. Please note that we do not name the buttons
/// left, right, and middle, as this assumes we use a mouse for right-hand people.
///
/// JS supports up to 5 mouse buttons currently:
/// https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button
#[derive(Debug,Clone,Copy)]
pub enum Button {_0,_1,_2,_3,_4}

impl Button {
    pub fn from_code(code:i16) -> Self {
        match code {
            0 => Self::_0,
            1 => Self::_1,
            2 => Self::_2,
            3 => Self::_3,
            4 => Self::_4,
            _ => panic!("Invalid button code"),
        }
    }
}
