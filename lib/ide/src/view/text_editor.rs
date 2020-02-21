//! This module contains TextEditor, an UiComponent to edit Enso Modules or Text Files.

use crate::prelude::*;

use crate::view::KeyboardClosure;
use crate::view::KeyboardListener;
use crate::view::temporary_panel::TemporaryPadding;
use crate::view::temporary_panel::TemporaryPanel;

use basegl::display::object::DisplayObjectOps;
use basegl::display::shape::text::glyph::font::FontRegistry;
use basegl::display::shape::text::text_field::TextField;
use basegl::display::shape::text::text_field::TextFieldProperties;
use basegl::display::world::*;

use nalgebra::Vector2;
use nalgebra::zero;
use web_sys::KeyboardEvent;



// ==================
// === TextEditor ===
// ==================

shared! { TextEditor

/// TextEditor allows us to edit text files or Enso Modules. Extensible code highlighting is
/// planned to be implemented for it.
#[derive(Debug)]
pub struct TextEditorData {
    text_field   : TextField,
    padding      : TemporaryPadding,
    position     : Vector2<f32>,
    size         : Vector2<f32>,
    controller   : controller::text::Handle,
    key_listener : Option<KeyboardListener>
}

impl {
    /// Updates the underlying display object.
    pub fn update(&self) {
        let padding  = self.padding;
        let position = self.position;
        let position = Vector3::new(position.x + padding.left, position.y + padding.bottom, 0.0);
        self.text_field.set_position(position);
        // TODO: set text field size once the property change will be supported.
        // let padding  = Vector2::new(padding.left + padding.right, padding.top + padding.bottom);
        // self.text_field.set_size(self.dimensions - padding);
        self.text_field.update();
    }

    /// Saves text editor's content to file.
    pub fn save(&self) {
        let controller = self.controller.clone();
        let text       = self.text_field.get_content();
        let store_fut  = controller.store_content(text);
        executor::global::spawn(async move {
            if store_fut.await.is_err() {
                println!("Failed to write file")
            }
        });
    }
}}

impl TextEditor {
    /// Creates a new TextEditor.
    pub fn new(world:&World, controller:controller::text::Handle) -> Self {
        let scene        = world.scene();
        let camera       = scene.camera();
        let screen       = camera.screen();
        let mut fonts    = FontRegistry::new();
        let font         = fonts.get_or_load_embedded_font("DejaVuSansMono").unwrap();
        let padding      = default();
        let position     = zero();
        let size         = Vector2::new(screen.width, screen.height);
        let black        = Vector4::new(0.0,0.0,0.0,1.0);
        let base_color   = black;
        let text_size    = 16.0;
        let properties   = TextFieldProperties {font,text_size,base_color,size};
        let text_field   = TextField::new(&world,properties);
        let key_listener = None;

        let content_future  = controller.read_content();
        let text_field_weak = text_field.downgrade();
        executor::global::spawn(async move {
            if let Ok(content) = content_future.await {
                if let Some(text_field) = text_field_weak.upgrade() {
                    text_field.write(&content);
                }
            }
        });
        world.add_child(&text_field);

        let data = TextEditorData {controller,text_field,padding,position,size,key_listener};
        Self::new_from_data(data).initialize()
    }

    fn initialize(self) -> Self {
        let text_editor = Rc::downgrade(&self.rc);
        let closure     = move |event:KeyboardEvent| {
            const S_KEY : u32 = 83;
            if event.ctrl_key() && event.key_code() == S_KEY {
                if let Some(text_editor) = text_editor.upgrade() {
                    text_editor.borrow().save();
                }
            }
        };
        let closure      = Box::new(closure);
        let callback     = KeyboardClosure::wrap(closure);
        let key_listener = KeyboardListener::new("keydown".into(), callback);
        self.rc.borrow_mut().key_listener = Some(key_listener);
        self.update();
        self
    }
}

impl TemporaryPanel for TextEditor {
    fn set_padding(&mut self, padding: TemporaryPadding) {
        self.rc.borrow_mut().padding = padding;
    }

    fn padding(&self) -> TemporaryPadding {
        self.rc.borrow().padding
    }

    fn set_size(&mut self, size:Vector2<f32>) {
        self.rc.borrow_mut().size = size;
        self.update();
    }

    fn size(&self) -> Vector2<f32> {
        self.rc.borrow_mut().text_field.size()
    }

    fn set_position(&mut self, position:Vector2<f32>) {
        self.rc.borrow_mut().position = position;
        self.update();
    }

    fn position(&self) -> Vector2<f32> {
        let position = self.rc.borrow().text_field.position();
        Vector2::new(position.x, position.y)
    }
}
