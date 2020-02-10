use super::text_editor::TextEditor;
use basegl::system::web::*;

use js_sys::Function;
use web_sys::KeyboardEvent;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use basegl::display::world::World;
use super::ui_component::UiComponent;

use nalgebra::zero;
use nalgebra::Vector2;
use std::rc::Rc;
use std::cell::RefCell;


//TODO: ViewMode is a temporary enumeration, it will be replaced by proper Panel impl.
// ================
// === ViewMode ===
// ================

pub enum ViewMode {
    Full,
    Half
}



// ======================
// === ViewLayoutData ===
// ======================

struct ViewLayoutData {
    text_editor      : TextEditor,
    keyboard_closure : Option<Closure<dyn FnMut(KeyboardEvent)>>,
    view_mode        : ViewMode,
    dimensions       : Vector2<f32>
}

impl Drop for ViewLayoutData {
    fn drop(&mut self) {
        self.keyboard_closure.as_ref().map(|keyboard_closure| {
            let body = document().unwrap().body().unwrap();
            let callback : &Function = keyboard_closure.as_ref().unchecked_ref();
            body.remove_event_listener_with_callback("keydown", callback).ok();
        });
    }
}

impl ViewLayoutData {
    fn set_view_mode(&mut self, view_mode:ViewMode) {
        self.view_mode = view_mode;
        self.update();
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
        self.update();
    }

    fn update(&mut self) {
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
        self.text_editor.set_dimensions(dimensions);
        self.text_editor.set_position(position);
        self.text_editor.update();
    }
}

// ==================
// === ViewLayout ===
// ==================

pub struct ViewLayout {
    data : Rc<RefCell<ViewLayoutData>>
}

impl ViewLayout {
    pub fn new(world:&World) -> Self {
        let text_editor      = TextEditor::new(&world);
        let keyboard_closure = None;
        let view_mode        = ViewMode::Half;
        let dimensions       = zero();
        let data             = ViewLayoutData{text_editor,keyboard_closure,view_mode,dimensions};
        let data             = Rc::new(RefCell::new(data));
        Self {data}.init(world)
    }

    fn init_keyboard(mut self) -> Self {
        let data    = Rc::downgrade(&self.data);
        let closure = move |event:KeyboardEvent| {
            const F_KEY : u32 = 70;
            if event.ctrl_key() && event.key_code() == F_KEY {
                data.upgrade().map(|data| {
                    data.borrow_mut().switch_mode()
                });
                event.prevent_default();
            }
        };
        let closure              = Box::new(closure) as Box<dyn FnMut(KeyboardEvent)>;
        let keyboard_closure     = Closure::wrap(closure);
        let callback : &Function = keyboard_closure.as_ref().unchecked_ref();
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

    pub fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.data.borrow_mut().set_dimensions(dimensions)
    }

    pub fn set_view_mode(&mut self, view_mode:ViewMode) {
        self.data.borrow_mut().set_view_mode(view_mode)
    }
}