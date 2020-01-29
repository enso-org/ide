pub mod content;
pub mod cursor;
pub mod fragment;
pub mod rendered;

use crate::prelude::*;

use crate::display::shape::text::content::{TextFieldContent, TextChange};
use crate::display::shape::text::cursor::Cursors;
use crate::display::shape::text::cursor::Step;
use crate::display::shape::text::cursor::CursorNavigation;

use nalgebra::{Vector2, Vector3, Vector4};
use crate::display::shape::glyph::font::{FontId, FontRegistry};
use crate::display::shape::text::rendered::RenderedContent;
use crate::display::object::DisplayObjectData;
use crate::display::shape::text::fragment::DisplayedLinesUpdate;


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
    rendered         : RenderedContent,
    display_object   : DisplayObjectData,
}

impl TextField {
    pub fn new(text:&str, text_size:f32, font_id:FontId, color:Vector4<f32>, viewport_size: Vector2<f32>, fonts:&mut FontRegistry) -> Self {
        let logger = Logger::new("TextField");
        let mut text_field = TextField {
            content: TextFieldContent::new(font_id,text,text_size),
            cursors: Cursors::new(),
            text_size,
            base_color: color,
            rendered:RenderedContent::new(viewport_size,text_size,color,font_id,fonts),
            viewport_size,
            display_object: DisplayObjectData::new(logger),
        };

        text_field.display_object.add_child(text_field.rendered.display_object.clone_ref());
        text_field.assignment_update(fonts).update_line_assignment();
        println!("{:?}", text_field.rendered.assignment.assignment[0]);
        text_field.rendered.display_object.set_position(Vector3::new(0.0,0.0,0.0));
        text_field.rendered.update(&mut text_field.content,fonts);
        text_field
    }

    pub fn set_position(&mut self, position:Vector3<f32>) {
        self.display_object.set_position(position);
    }

    /// Scroll text by given offset.
    ///
    /// The value of 1.0 on both dimensions is equal to one line's height.
    pub fn scroll(&mut self, offset:Vector2<f32>, fonts:&mut FontRegistry) {
        self.rendered.display_object.mod_position(|pos| *pos -= Vector3::new(offset.x,offset.y,0.0));
        let mut update = self.assignment_update(fonts);
        if offset.x != 0.0 {
            update.update_after_x_scroll(offset.x);
        }
        if offset.y != 0.0 {
            update.update_line_assignment();
        }
        self.rendered.update(&mut self.content,fonts);
    }

    /// Get current scroll position.
    pub fn scroll_position(&self) -> Vector2<f32> {
        self.rendered.display_object.position().xy()
    }

//    /// Jump to scroll position.
//    pub fn jump_to_position(&mut self, scroll_position:Vector2<f64>) {
//        self.rendered.jump_to(scroll_position);
//    }

    pub fn navigate_cursors(&mut self, step:Step, selecting:bool, fonts:&mut FontRegistry) {
        let content        = self.content.full_info(fonts);
        let mut navigation = CursorNavigation {content,selecting};
        self.cursors.navigate_all_cursors(&mut navigation,&step);
    }

    pub fn make_change(&mut self, change:TextChange, fonts:&mut FontRegistry) {
        self.content.make_change(change);
        self.assignment_update(fonts).update_after_text_edit();
        self.rendered.update(&mut self.content,fonts);
    }

    fn assignment_update<'a,'b>(&'a mut self, fonts:&'b mut FontRegistry)
    -> DisplayedLinesUpdate<'a,'b,'a> {
        DisplayedLinesUpdate {
            content: self.content.full_info(fonts),
            assignment: &mut self.rendered.assignment,
            scroll_offset: -self.rendered.display_object.position().xy(),
            view_size: self.viewport_size,
        }
    }

    pub fn update(&self) {
        self.display_object.update()
    }
}

impl From<&TextField> for DisplayObjectData {
    fn from(text_fields: &TextField) -> Self {
        text_fields.display_object.clone_ref()
    }
}
