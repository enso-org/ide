use crate::prelude::*;

use crate::*;
use rust_dense_bitset::BitSet;
use rust_dense_bitset::DenseBitSetExtended;





// ===============
// === KeyMask ===
// ===============

const MAX_KEY_CODE : usize = 127;

#[derive(Clone,Debug,Eq,PartialEq,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct KeyMask(pub DenseBitSetExtended);

impl Default for KeyMask {
    fn default() -> Self {
        Self(DenseBitSetExtended::with_capacity(MAX_KEY_CODE + 1))
    }
}

impl FromIterator<u8> for KeyMask {
    fn from_iter<T: IntoIterator<Item=u8>>(iter: T) -> Self {
        let mut key_mask = KeyMask::default();
        for bit in iter { key_mask.set_bit(bit as usize,true); }
        key_mask
    }
}



// ================
// === KeyState ===
// ================

#[derive(Copy,Clone,Debug,Default)]
struct KeyState {
    code    : u8,
    pressed : bool,
}

impl KeyState {
    fn key_pressed(code:&u8) -> Self {
        let pressed = true;
        let code    = *code;
        KeyState{code,pressed}
    }

    fn key_released(code:&u8) -> Self {
        let pressed = false;
        let code    = *code;
        KeyState{code,pressed}
    }

    fn updated_mask(&self, mask:&KeyMask) -> KeyMask {
        let mut mask = mask.clone();
        mask.set_bit(self.code as usize, self.pressed);
        mask
    }
}




// ================
// === Keyboard ===
// ================

/// Keyboard FRP bindings.
#[derive(Debug)]
pub struct Keyboard {
    /// The mouse up event.
    pub key_pressed: Dynamic<u8>,
    /// The mouse down event.
    pub key_released: Dynamic<u8>,
    pub key_mask: Dynamic<KeyMask>,
}

impl Default for Keyboard {
    fn default() -> Self {
        frp! {
            keyboard.key_pressed        = source();
            keyboard.key_released       = source();
            keyboard.key_pressed_state  = key_pressed.map(KeyState::key_pressed);
            keyboard.key_released_state = key_released.map(KeyState::key_released);
            keyboard.key_state          = key_pressed_state.merge(&key_released_state);
            keyboard.previous_key_mask  = recursive::<KeyMask>();
            keyboard.key_mask           = key_state.map2(&previous_key_mask,KeyState::updated_mask);

        }
        previous_key_mask.initialize(&key_mask);

        Keyboard {key_pressed,key_released,key_mask}
    }
}



// =======================
// === KeyboardActions ===
// =======================

pub struct

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_mask() {
        let keyboard                  = Keyboard::default();
        let expected_key_mask:KeyMask = default();
        assert_eq!(expected_key_mask, keyboard.key_mask.behavior.current_value());

        keyboard.key_pressed.event.emit(13);
        let expected_key_mask:KeyMask = std::iter::once(13).collect();
        assert_eq!(expected_key_mask, keyboard.key_mask.behavior.current_value());

        keyboard.key_pressed.event.emit(15);
        let expected_key_mask:KeyMask = [13,15].iter().cloned().collect();
        assert_eq!(expected_key_mask, keyboard.key_mask.behavior.current_value());

        keyboard.key_released.event.emit(13);
        let expected_key_mask:KeyMask = std::iter::once(15).collect();
        assert_eq!(expected_key_mask, keyboard.key_mask.behavior.current_value());
    }
}