//! This module contains implementation of ViewLayout with a single TextEditor temporarily
//! occupying half bottom of the screen temporarily as the default layout.

use wasm_bindgen::prelude::*;

use super::text_editor::TextEditor;
use super::ui_component::UiComponent;

use basegl::system::web::*;
use basegl::display::world::World;

use nalgebra::zero;
use nalgebra::Vector2;
use std::rc::Rc;
use std::cell::RefCell;
use js_sys::Function;
use web_sys::KeyboardEvent;
use wasm_bindgen::JsCast;
use crate::view::ui_component::Padding;


//TODO: ViewMode is a temporary enumeration, it will be replaced by proper Panel impl.
// ================
// === ViewMode ===
// ================

/// Defines the element's view mode. It can fully occupy the screen or only half of it.
#[derive(Clone,Copy,Debug)]
enum ViewMode {
    Full,
    Half
}



// ======================
// === ViewLayoutData ===
// ======================

#[derive(Debug)]
struct ViewLayoutData {
    text_editor      : TextEditor,
    keyboard_closure : Option<Closure<dyn FnMut(KeyboardEvent)>>,
    view_mode        : ViewMode,
    dimensions       : Vector2<f32>
}

impl Drop for ViewLayoutData {
    fn drop(&mut self) {
        if let Some(keyboard_closure) = self.keyboard_closure.as_ref() {
            let body = document().unwrap().body().unwrap();
            let callback : &Function = keyboard_closure.as_ref().unchecked_ref();
            body.remove_event_listener_with_callback("keydown", callback).ok();
        }
    }
}

impl ViewLayoutData {
    fn set_view_mode(&mut self, view_mode:ViewMode) {
        self.view_mode = view_mode;
        self.recalculate_layout();
    }

    fn switch_mode(&mut self) {
        if let ViewMode::Half = self.view_mode {
            self.set_view_mode(ViewMode::Full)
        } else {
            self.set_view_mode(ViewMode::Half)
        }
    }

    fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.dimensions = dimensions;
        self.recalculate_layout();
    }

    fn recalculate_layout(&mut self) {
        let dimensions = self.dimensions;
        let (position,dimensions) = match self.view_mode {
            ViewMode::Full => {
                let position   = Vector2::new(0.0, dimensions.y);
                (position,dimensions)
            },
            ViewMode::Half => {
                let position   = Vector2::new(0.0, dimensions.y / 2.0);
                let dimensions = Vector2::new(dimensions.x, dimensions.y / 2.0);
                (position,dimensions)
            }
        };
        self.text_editor.set_padding(Padding::new(10.0, 10.0, 10.0, 10.0));
        self.text_editor.set_dimensions(dimensions);
        self.text_editor.set_position(position);
        self.text_editor.update();
    }
}



// ==================
// === ViewLayout ===
// ==================

/// Initial implementation of ViewLayout with a single TextEditor. Pressing ctrl+f toggles
/// fullscreen mode.
#[derive(Debug,Clone)]
pub struct ViewLayout {
    data : Rc<RefCell<ViewLayoutData>>
}

impl ViewLayout {
    /// Creates a new ViewLayout with a single TextEditor.
    pub fn default(world:&World) -> Self {
        let text_editor      = TextEditor::new(&world);
        let keyboard_closure = None;
        let view_mode        = ViewMode::Half;
        let dimensions       = zero();
        let data             = ViewLayoutData {text_editor,keyboard_closure,view_mode,dimensions};
        let data             = Rc::new(RefCell::new(data));
        Self {data}.init(world)
    }

    fn init_keyboard(self) -> Self {
        let data    = Rc::downgrade(&self.data);
        let closure = move |event:KeyboardEvent| {
            const F_KEY : u32 = 70;
            if event.ctrl_key() && event.key_code() == F_KEY {
                if let Some(data) = data.upgrade() {
                    data.borrow_mut().switch_mode()
                }
                event.prevent_default();
            }
        };
        let closure : Box<dyn FnMut(KeyboardEvent)> = Box::new(closure);
        let keyboard_closure                        = Closure::wrap(closure);
        let callback : &Function                    = keyboard_closure.as_ref().unchecked_ref();
        let body = document().unwrap().body().unwrap();
        body.add_event_listener_with_callback("keydown", callback).ok();
        self.data.borrow_mut().keyboard_closure = Some(keyboard_closure);
        self
    }

    fn init(mut self, world:&World) -> Self {
        let screen = world.scene().camera().screen();
        let dimensions = Vector2::new(screen.width,screen.height);
        self.set_dimensions(dimensions);
        self.init_keyboard()
    }

    /// Sets dimensions.
    pub fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.data.borrow_mut().set_dimensions(dimensions)
    }
}
