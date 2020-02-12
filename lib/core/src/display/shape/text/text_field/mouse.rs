//! A FRP definitions for mouse event handling, with biding this FRP graph to js events.

use crate::prelude::*;

use crate::control::io::mouse2::event::*;
use crate::control::io::mouse2::MouseManager;
use crate::display::shape::text::text_field::keyboard::TextFieldKeyboardFrp;
use crate::display::shape::text::text_field::TextFieldData;
use crate::system::web;

use enso_frp::*;
use nalgebra::Vector2;



/// All nodes of FRP graph related to TextField operations.
#[derive(Debug)]
pub struct TextFieldMouseFrp {
    /// A "Mouse" common part of this graph from FRP library.
    pub mouse: Mouse,
    /// Event emitted on click inside the TextField.
    pub click_in: Dynamic<()>,
    /// Node giving `true` value during selection (clicked inside TextField and keeping pressed).
    pub selecting: Dynamic<bool>,
    /// Node giving `true` when using keyboard modifiers for multicursor edit.
    pub multicursor: Dynamic<bool>,
    /// A node setting cursor after mouse click.
    pub set_cursor_action: Dynamic<()>,
    /// A node modifying selection on mouse drag.
    pub select_action: Dynamic<()>,
}

impl TextFieldMouseFrp {
    /// Create FRP graph doing actions on given TextField.
    pub fn new(text_field_ptr:Weak<RefCell<TextFieldData>>, keyboard:&TextFieldKeyboardFrp)
    -> Self {
        use Key::*;
        let mouse               = Mouse::default();
        let is_inside           = Self::is_inside_text_field_lambda(text_field_ptr.clone());
        let is_multicursor_mode = |mask:&KeyMask| mask == &[Shift].iter().collect();
        let set_cursor_action   = Self::set_cursor_lambda(text_field_ptr.clone());
        let select_action       = Self::select_lambda(text_field_ptr.clone());
        frp! {
            text_field.is_inside     = mouse.position.map(is_inside);
            text_field.click_in      = mouse.down.gate(&is_inside);
            text_field.click_in_bool = click_in.constant(true);
            text_field.mouse_up_bool = mouse.up.constant(false);
            text_field.selecting     = click_in_bool.merge(&mouse_up_bool);
            text_field.multicursor   = keyboard.keyboard.key_mask.map(is_multicursor_mode);

            text_field.click_in_pos = mouse.position.sample(&click_in);
            text_field.select_pos   = mouse.position.gate(&selecting);

            text_field.set_cursor_action = click_in_pos.map2(&multicursor,set_cursor_action);
            text_field.select_action     = select_pos.map(select_action);
        }
        Self {mouse,click_in,selecting,multicursor,set_cursor_action,select_action}
    }

    /// Bind this FRP graph to js events.
    pub fn bind_frp_to_mouse(&self) -> MouseManager  {
        let mouse_manager = MouseManager::new(&web::document().unwrap());
        let height        = web::window().inner_height().unwrap().as_f64().unwrap() as i32;
        let frp_position  = self.mouse.position.event.clone_ref();
        let frp_down      = self.mouse.down.event.clone_ref();
        let frp_up        = self.mouse.up.event.clone_ref();
        let handle = mouse_manager.on_move.add(move |event:&OnMove| {
            frp_position.emit(Position::new(event.client_x(),height - event.client_y()));
        });
        handle.forget();
        let handle = mouse_manager.on_down.add(move |_:&OnDown| {
            frp_down.emit(());
        });
        handle.forget();
        let handle = mouse_manager.on_up.add(move |_:&OnUp| {
            frp_up.emit(());
        });
        handle.forget();
        mouse_manager
    }
}

// === Private functions ===

impl TextFieldMouseFrp {
    fn is_inside_text_field_lambda(text_field_ptr:Weak<RefCell<TextFieldData>>)
    -> impl Fn(&Position) -> bool {
        move |position| {
            let position = Vector2::new(position.x as f32,position.y as f32);
            text_field_ptr.upgrade().map_or(false, |tf| tf.borrow().is_inside(position))
        }
    }

    fn set_cursor_lambda(text_field_ptr:Weak<RefCell<TextFieldData>>)
    -> impl Fn(&Position,&bool) {
        move |position,multicursor| {
            let position = Vector2::new(position.x as f32,position.y as f32);
            if let Some(text_field) = text_field_ptr.upgrade() {
                if *multicursor {
                    text_field.borrow_mut().add_cursor(position);
                } else {
                    text_field.borrow_mut().set_cursor(position);
                }
            }
        }
    }

    fn select_lambda(text_field_ptr:Weak<RefCell<TextFieldData>>) -> impl Fn(&Position) {
        move |position| {
            let position = Vector2::new(position.x as f32,position.y as f32);
            if let Some(text_field) = text_field_ptr.upgrade() {
                text_field.borrow_mut().jump_cursor(position,true);
            }
        }
    }
}