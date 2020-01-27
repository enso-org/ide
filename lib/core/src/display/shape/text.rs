pub mod content;
pub mod cursor;
pub mod fragment;
pub mod rendered;

use crate::prelude::*;

use crate::display::Scene;
use crate::system::gpu::shader::Context;
use crate::display::shape::text::buffer::RenderedLines;
use crate::display::shape::text::content::TextFieldContent;
use crate::display::shape::text::cursor::Cursors;
use crate::display::shape::text::cursor::Step;
use crate::display::shape::text::cursor::CursorNavigation;
use crate::display::shape::text::font::FontId;
use crate::display::shape::text::font::Fonts;
use crate::display::shape::text::msdf::MsdfTexture;
use crate::display::shape::text::program::MsdfProgram;
use crate::display::shape::text::program::BasicProgram;
use crate::display::shape::text::program::create_content_program;
use crate::display::shape::text::program::create_cursors_program;

use nalgebra::{Vector2, Vector4};
use nalgebra::Similarity2;
use nalgebra::Point2;
use nalgebra::Projective2;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlTexture;
use crate::display::shape::glyph::font::FontId;
use crate::display::shape::text::rendered::RenderedContent;
use crate::display::object::DisplayObjectData;


// =====================
// === TextComponent ===
// =====================


/// Component rendering text
///
/// This component is under heavy construction, so the api may easily changed in few future
/// commits.
#[derive(Debug)]
pub struct TextField {
    pub content      : TextFieldContent,
    pub cursors      : Cursors,
    pub text_size    : f32,
    pub base_color   : Vector4<f32>,
    pub viewport_size: Vector2<f32>,
    scroll_offset    : Vector2<f32>,
    rendered         : RenderedContent,
    display_object   : DisplayObjectData,
}

impl TextField {
    pub fn new(text:&str, text_size:f32, font_id:FontId, color:Vector4<f32>, viewport_size: Vector2<f32>, logger:Logger) -> Self {
        TextField {
            content: TextFieldContent::new(font_id,text,text_size),
            cursors: Cursors::new(),
            text_size,
            base_color: color,
            rendered: RenderedContent::new(viewport_size,text_size,color,font_id),
            viewport_size,
            scroll_offset: Vector2::new(0.0,0.0),
            display_object: DisplayObjectData::new(logger),
        }
    }

    /// Scroll text by given offset.
    ///
    /// The value of 1.0 on both dimensions is equal to one line's height.
    pub fn scroll(&mut self, offset:Vector2<f64>) {
        self.rendered.scroll(offset);
    }

    /// Get current scroll position.
    ///
    /// The _scroll_position_ is a position of top-left corner of the first line.
    /// The offset of 1.0 on both dimensions is equal to one line's height.
    pub fn scroll_position(&self) -> &Vector2<f64> {
        &self.rendered.window_offset
    }

    /// Jump to scroll position.
    ///
    /// The `scroll_position` is a position of top-left corner of the first line.
    /// The offset of 1.0 on both dimensions is equal to one line's height.
    pub fn jump_to_position(&mut self, scroll_position:Vector2<f64>) {
        self.rendered.jump_to(scroll_position);
    }

    pub fn navigate_cursors(&mut self, step:Step, selecting:bool, fonts:&mut Fonts) {
        let content        = &mut self.content;
        let mut navigation = CursorNavigation {content,fonts,selecting};
        self.cursors.navigate_all_cursors(&mut navigation,step);
    }
}
