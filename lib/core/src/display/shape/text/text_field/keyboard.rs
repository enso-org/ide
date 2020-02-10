use crate::prelude::*;
use enso_frp::*;
use enso_frp::io::keyboard::{KeyboardActions, Keyboard, Key};
use crate::system::web::text_input::KeyboardBinding;
use web_sys::KeyboardEvent;



// ====================
// === TextFieldFrp ===
// ====================

/// This structure contains all nodes in FRP graph handling keyboards events of one TextField
/// component.
///
/// The most of TextField actions are covered by providing actions to KeyboardActions for specific
/// key masks. However, there are special actions which must be done in a lower level:
///  * *clipboard operations* - they are performed by reading text input js events directly from
///    text area component. See `system::web::text_input` crate.
///  * *text input operations* - here we want to handle all the keyboard mapping set by user, so
///    we connect this action directly to `key_press` node from `keyboard`.
pub struct TextFieldFrp {
    /// A "keyboard" part of graph derived from frp crate.
    keyboard: Keyboard,
    /// Keyboard actions. Here we define shortcuts for all actions except letters input, copying
    /// and pasting.
    actions: KeyboardActions,
    /// A node producing event once cut operation was requested.
    cut: Dynamic<()>,
    /// A node producing event once copy operation was requested.
    copy: Dynamic<()>,
    /// A node producing event once paste operation was requested.
    paste: Dynamic<String>,
    /// A lambda node performing cut operation. Returns the string which should be copied to
    /// clipboard.
    cut_action: Dynamic<String>,
    /// A lambda node performing copy operation. Returns the string which should be copied to
    /// clipboard.
    copy_action: Dynamic<String>,
    /// A lambda node performing paste operation.
    paste_action: Dynamic<()>,
    /// A lambda node performing character input operation.
    char_typed_action: Dynamic<()>,
}

impl TextFieldFrp {
    /// Create FRP graph operating on given TextField pointer.
    pub fn new(text_field_ptr:Rc<RefCell<TextFieldData>>) -> TextFieldFrp {
        let keyboard          = Keyboard::default();
        let actions           = KeyboardActions::new(&keyboard);
        let cut_action        = Self::copy_action_lambda(true,text_field_ptr.downgrade());
        let copy_action       = Self::copy_action_lambda(false,text_field_ptr.downgrade());
        let paste_action      = Self::paste_action_lambda(text_field_ptr.downgrade());
        let char_typed_action = Self::char_typed_lambda(text_field_ptr.downgrade());
        frp! {
            text_field.cut              = source();
            text_field.copy             = source();
            text_field.paste            = source();
            text_field.copy_action      = copy.map(copy_action);
            text_field.cut_action       = cut.map(cut_action);
            text_field.paste_action     = paste.map(paste_action);
            text_field.key_typed_action = keyboard.key_pressed.map(char_typed_action);
        }
        TextFieldFrp
            {keyboard,actions,cut,copy,paste,cut_action,copy_action,paste_action,char_typed_action}
    }

    /// Bind this FRP graph to js events.
    ///
    /// Until the returned `KeyboardBinding` structure lives, the js events will emit the proper
    /// source events in this graph.
    pub fn bind_frp_to_js_text_input_actions(frp:&TextFieldFrp) -> KeyboardBinding {
        let mut binding      = KeyboardBinding::create();
        let frp_key_pressed  = frp.keyboard.key_pressed.clone_ref();
        let frp_key_released = frp.keyboard.key_released.clone_ref();
        let frp_copy         = frp.copy.clone_ref();
        let frp_paste        = frp.paste.clone_ref();
        let frp_text_to_copy = frp.copy_action.clone_ref();
        binding.set_key_up_handler(|event| {
            if Ok(key) = key_from_event(&event) {
                frp_key_pressed.event.emit(key);
            }
        });
        binding.set_key_down_handler(|event| {
            if Ok(key) = key_from_event(&event) {
                frp_key_released.event.emit(key);
            }
        });
        binding.set_copy_handler(|| {
            frp_copy.event.emit();
            frp_text_to_copy.behavior.current_value()
        });
        binding.set_paste_handler(|text_to_paste| {
            frp_paste.event.emit(text_to_paste);
        });
        binding
    }
}


// === Private ===

impl TextFieldFrp {

    fn copy_action_lambda(cut:bool, text_field_ptr:Weak<RefCell<TextFieldData>>)
    -> impl FnMut() -> String {
        || {
            match text_field_ptr.upgrade() {
                Some(text_field) => {
                    let text_field_ref = text_field.borrow_mut();
                    let result = text_field_ref.get_selected_text();
                    if cut { text_field_ref.edit(""); }
                    result
                },
                None => default()
            }
        }
    }

    fn paste_action_lambda(text_field_ptr:Weak<RefCell<TextFieldData>>) -> impl FnMut(String) {
        |text_to_paste| {
            text_field_ptr.upgrade().for_each(|text_field| { text_field.edit(pasted.as_str()) })
        }
    }

    fn char_typed_lambda(text_field_ptr:Weak<RefCell<TextFieldData>>) -> impl FnMut(&Key) {
        |key| {
            text_field_ptr.upgrade().for_each(|text_field| {
                if let Key::Character(string) = key {
                    text_field.edit(string);
                }
            })
        }
    }

    fn initialize_actions_map(actions:&mut KeyboardActions, text_field_ptr:Weak<RefCell<TextFieldData>>)
}

fn key_from_event(event:&KeyboardEvent) -> Result<Key,FromStr::Err> {
    let key:String                     = event.key();
    let first_chars:SmallVec<[char;2]> = key.chars().take(2).collect();
    if first_chars.len() == 1 {
        Ok(Key::Character(key))
    } else {
        key.parse()
    }
}
