use crate::prelude::*;

use crate::*;
use rust_dense_bitset::BitSet;
use rust_dense_bitset::DenseBitSetExtended;
use keyboard_types::Key;
use std::collections::hash_map::Entry;


// ===============
// === KeyMask ===
// ===============

const MAX_KEY_CODE : usize = 255;

#[derive(BitXor,Clone,Debug,Eq,Hash,PartialEq,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct KeyMask(pub DenseBitSetExtended);

impl Default for KeyMask {
    fn default() -> Self {
        let mut bitset = DenseBitSetExtended::with_capacity(MAX_KEY_CODE + 1);
        // This is the only way to set bitset length.
        bitset.set_bit(MAX_KEY_CODE, false);
        Self(bitset)
    }
}

impl FromIterator<Key> for KeyMask {
    fn from_iter<T: IntoIterator<Item=Key>>(iter:T) -> Self {
        let mut key_mask = KeyMask::default();
        for key in iter {
            let bit = key.legacy_keycode() as usize;
            key_mask.set_bit(bit,true);
        }
        key_mask
    }
}



// ================
// === KeyState ===
// ================

#[derive(Clone,Debug,Default)]
struct KeyState {
    key     : Key,
    pressed : bool,
}

impl KeyState {
    fn key_pressed(key:&Key) -> Self {
        let pressed = true;
        let key    = key.clone();
        KeyState{key,pressed}
    }

    fn key_released(key:&Key) -> Self {
        let pressed = false;
        let key     = key.clone();
        KeyState{key,pressed}
    }

    fn updated_mask(&self, mask:&KeyMask) -> KeyMask {
        let mut mask = mask.clone();
        let bit      = self.key.legacy_keycode() as usize;
        mask.set_bit(bit,self.pressed);
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
    pub key_pressed: Dynamic<Key>,
    /// The mouse down event.
    pub key_released: Dynamic<Key>,
    /// The structure holding mask of all of the currently pressed keys.
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

pub trait Action    = FnMut(&KeyMask) + 'static;
pub type  ActionMap = HashMap<KeyMask,Box<dyn Action>>;

pub struct KeyboardActions {
    action_map: Rc<RefCell<ActionMap>>,
    action: Dynamic<()>,
}

impl KeyboardActions {
    fn new(keyboard:&Keyboard) -> Self {
        let action_map = Rc::new(RefCell::new(HashMap::new()));
        frp! {
            keyboard.action = keyboard.key_mask.map(Self::perform_action_lambda(action_map.clone()));
        }
        KeyboardActions{action_map,action}
    }

    fn perform_action_lambda(action_map:Rc<RefCell<ActionMap>>) -> impl Fn(&KeyMask) {
        move |key_mask| {
            if let Some((map_mask, mut action)) = with(action_map.borrow_mut(), |mut map| map.remove_entry(key_mask)) {
                action(key_mask);
                if let Entry::Vacant(entry) =  action_map.borrow_mut().entry(map_mask) {
                    entry.insert(action);
                }
            }
        }
    }

    fn set_action<F:FnMut(&KeyMask) + 'static>(&mut self, key_mask:KeyMask, action:F) {
        self.action_map.borrow_mut().insert(key_mask,Box::new(action));
    }

    fn unset_action(&mut self, key_mask:&KeyMask) {
        self.action_map.borrow_mut().remove(key_mask);
    }
}




// =================
// === TextInput ===
// =================

#[derive(Debug)]
pub struct TextInput {
    pub keyboard: Keyboard,
    pub copy: Dynamic<()>,
    pub paste: Dynamic<String>,
    pub text_to_copy: Dynamic<String>,
}

impl TextInput {

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_mask() {
        let keyboard                  = Keyboard::default();
        let expected_key_mask:KeyMask = default();
        assert_eq!(expected_key_mask, keyboard.key_mask.behavior.current_value());
        let key1 = Key::Character("x".to_string());
        let key2 = Key::Control;

        keyboard.key_pressed.event.emit(key1.clone());
        let expected_key_mask:KeyMask = std::iter::once(key1.clone()).collect();
        assert_eq!(expected_key_mask, keyboard.key_mask.behavior.current_value());

        keyboard.key_pressed.event.emit(key2.clone());
        let expected_key_mask:KeyMask = [key1.clone(),key2.clone()].iter().cloned().collect();
        assert_eq!(expected_key_mask, keyboard.key_mask.behavior.current_value());

        keyboard.key_released.event.emit(key1.clone());
        let expected_key_mask:KeyMask = std::iter::once(key2.clone()).collect();
        assert_eq!(expected_key_mask, keyboard.key_mask.behavior.current_value());
    }

    #[test]
    fn key_actions() {
        let undone            = Rc::new(RefCell::new(false));
        let undone1           = undone.clone();
        let redone            = Rc::new(RefCell::new(false));
        let redone1           = redone.clone();
        let undo_keys:KeyMask = [Key::Control, Key::Character("z".to_string())].iter().cloned().collect();
        let redo_keys:KeyMask = [Key::Control, Key::Character("y".to_string())].iter().cloned().collect();

        let keyboard    = Keyboard::default();
        let mut actions = KeyboardActions::new(&keyboard);
        actions.set_action(undo_keys.clone(), move |_| { *undone1.borrow_mut() = true });
        actions.set_action(redo_keys.clone(), move |_| { *redone1.borrow_mut() = true });
        keyboard.key_pressed.event.emit(Key::Character("Z".to_string()));
        assert!(!*undone.borrow());
        assert!(!*redone.borrow());
        keyboard.key_pressed.event.emit(Key::Control);
        assert!( *undone.borrow());
        assert!(!*redone.borrow());
        *undone.borrow_mut() = false;
        keyboard.key_released.event.emit(Key::Character("z".to_string()));
        assert!(!*undone.borrow());
        assert!(!*redone.borrow());
        keyboard.key_pressed.event.emit(Key::Character("y".to_string()));
        assert!(!*undone.borrow());
        assert!( *redone.borrow());
        *redone.borrow_mut() = false;
        keyboard.key_released.event.emit(Key::Character("y".to_string()));
        keyboard.key_released.event.emit(Key::Control);

        actions.unset_action(&undo_keys);
        keyboard.key_pressed.event.emit(Key::Character("Z".to_string()));
        keyboard.key_pressed.event.emit(Key::Control);
        assert!(!*undone.borrow());
        assert!(!*redone.borrow());
    }
}