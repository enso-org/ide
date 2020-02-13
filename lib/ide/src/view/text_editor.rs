//! This module contains TextEditor, an UiComponent to edit Enso Modules or Text Files.

use crate::prelude::*;

use basegl::display::object::DisplayObjectOps;
use basegl::display::shape::text::glyph::font::FontRegistry;
use basegl::display::shape::text::text_field::TextField;
use basegl::display::shape::text::text_field::TextFieldProperties;
use basegl::display::world::*;

use super::ui_component::UiComponent;

use nalgebra::Vector2;
use nalgebra::Vector4;



// =============
// === Color ===
// =============

//TODO[dg]:Move this to a better place.
mod color {
    use nalgebra::Vector4;

    pub type Color = Vector4<f32>;

    const BLACK : Color = Color::new(0.0, 0.0, 0.0, 1.0);
}




// ==================
// === TextEditor ===
// ==================

/// TextEditor allows us to edit text files or Enso Modules. Extensible code highlighting is
/// planned to be implemented for it.
#[derive(Clone,Debug)]
pub struct TextEditor {
    text_field : TextField
}

impl TextEditor {
    /// Creates a new TextEditor.
    pub fn new(world:&World) -> Self {
        let scene        = world.scene();
        let camera       = scene.camera();
        let screen       = camera.screen();
        let mut fonts    = FontRegistry::new();
        let font         = fonts.get_or_load_embedded_font("DejaVuSansMono").unwrap();

        let properties = TextFieldProperties {
            font,
            text_size  : 16.0,
            base_color : BLACK,
            size       : Vector2::new(screen.width, screen.height)
        };

        let text_field = TextField::new(&world,properties);
        text_field.set_position(Vector3::new(0.0, screen.height, 0.0));
        world.add_child(&text_field);
        text_field.update();

        Self {text_field}
    }

    /// Updates the underlying display object.
    pub fn update(&self) {
        self.text_field.update();
    }
}

impl UiComponent for TextEditor {
    fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.text_field.set_size(dimensions);
    }

    fn dimensions(&self) -> Vector2<f32> {
        self.text_field.size()
    }

    fn set_position(&mut self, position:Vector2<f32>) {
        self.text_field.set_position(Vector3::new(position.x, position.y, 0.0));
    }

    fn position(&self) -> Vector2<f32> {
        let position = self.text_field.position();
        Vector2::new(position.x, position.y)
    }
}
