use crate::prelude::*;

use crate::control::io::mouse2::event::*;
use crate::control::io::mouse2::MouseManager;
use crate::display::shape::text::text_field::keyboard::TextFieldKeyboardFrp;
use crate::display::shape::text::text_field::TextFieldData;
use crate::system::web;

use enso_frp::*;
use nalgebra::Vector2;



#[derive(Debug)]
pub struct TextFieldMouseFrp {
    pub mouse: Mouse,
    pub click_in: Dynamic<()>,
    pub selecting: Dynamic<bool>,
    pub multicursor: Dynamic<bool>,
    pub set_cursor_action: Dynamic<()>,
    pub select_action: Dynamic<()>,
}

impl TextFieldMouseFrp {
    pub fn new(text_field_ptr:Weak<RefCell<TextFieldData>>, keyboard:&TextFieldKeyboardFrp)
    -> Self {
        use Key::*;
        let mouse               = Mouse::default();
        let is_inside           = Self::is_inside_text_field_lambda(text_field_ptr.clone());
        let is_multicursor_mode = |mask:&KeyMask| mask == &[Alt,Shift].iter().collect();
        let set_cursor_action   = Self::set_cursor_lambda(text_field_ptr.clone());
        let select_action       = Self::select_lambda(text_field_ptr.clone());
        frp! {
            text_field.is_inside        = mouse.position.map(is_inside);
            text_field.click_in         = mouse.down.gate(&is_inside);
            text_field.click_in_bool    = click_in.constant(true);
            text_field.mouse_up_bool    = mouse.up.constant(false);
            text_field.selecting        = click_in_bool.merge(&mouse_up_bool);
            text_field.multicursor      = keyboard.keyboard.key_mask.map(is_multicursor_mode);

            text_field.click_in_pos     = mouse.position.sample(&click_in);
            text_field.select_pos       = mouse.position.gate(&selecting);

            text_field.set_cursor_action = click_in_pos.map2(&multicursor,set_cursor_action);
            text_field.select_action     = select_pos.map(select_action);
        }
        Self {mouse,click_in,selecting,multicursor,set_cursor_action,select_action}
    }

    /// Bind this FRP graph to js events.
    pub fn bind_frp_to_mouse(&self) -> MouseManager  {
        let mouse_manager = MouseManager::new(&web::document().unwrap());
        let frp_position  = self.mouse.position.event.clone_ref();
        let frp_down      = self.mouse.down.event.clone_ref();
        let frp_up        = self.mouse.up.event.clone_ref();
        let handle = mouse_manager.on_move.add(move |event:&OnMove| {
            frp_position.emit(Position::new(event.client_x(),event.client_y()));
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

impl TextFieldMouseFrp {
    fn is_inside_text_field_lambda(text_field_ptr:Weak<RefCell<TextFieldData>>)
    -> impl Fn(&Position) -> bool {
        move |position| {
            let position = Vector2::new(position.x as f32,position.y as f32);
            match text_field_ptr.upgrade() {
                Some(text_field) => text_field.borrow().is_inside(position),
                None             => false
            }
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