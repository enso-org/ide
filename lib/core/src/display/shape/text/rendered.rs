use crate::prelude::*;

use crate::display::shape::text::content::{TextFieldContent, TextFieldContentFullInfo};
use crate::display::shape::glyph::font::{FontRegistry, FontId};

use crate::display::shape::glyph::system::GlyphSystem;
use nalgebra::{Vector4, Vector3, Vector2};
use crate::display::shape::text::fragment::LineRenderersAssignment;
use crate::display::object::DisplayObjectData;
use crate::display::shape::primitive::system::ShapeSystem;
use crate::display::shape::text::cursor::{Cursors, Cursor};
use crate::display::shape::primitive::def::*;
use crate::display::symbol::geometry::compound::sprite::Sprite;

#[derive(Debug)]
pub struct CursorSprites {
    pub cursor    : Sprite,
    pub selection : Vec<Sprite>,
}


// ======================
// === Rendered Lines ===
// ======================

type GlyphLine = crate::display::shape::glyph::system::Line;

#[derive(Debug)]
pub struct RenderedContent {
    pub glyph_system                  : GlyphSystem,
    pub cursor_system                 : ShapeSystem,
    pub selection_system              : ShapeSystem,
    pub glyph_lines                   : Vec<GlyphLine>,
    pub cursors                       : Vec<CursorSprites>,
    pub assignment                    : LineRenderersAssignment,
    pub line_height                   : f32,
    pub display_object                : DisplayObjectData,
}

impl RenderedContent {

    /// Create the unassigned fragments for each displayed line.
    pub fn new(window_size:Vector2<f32>, line_height:f32, color:Vector4<f32>, font_id:FontId, fonts:&mut FontRegistry) -> RenderedContent {
        let cursor_definition = SharpRect("fract(input_time / 1000.0) < 0.5 ? 2.0 : 0.0", line_height);
        let selection_definition = RoundedRectByCorner("input_size.x", "input_size.y", 5.0, 5.0, 5.0, 5.0);
        let cursor_system     = ShapeSystem::new(&cursor_definition);
        let selection_system  = ShapeSystem::new(&selection_definition);
        let cursors           = Vec::new();
        let mut glyph_system  = GlyphSystem::new(font_id);
        let display_object    = DisplayObjectData::new(Logger::new("RenderedContent"));
        display_object.add_child(&glyph_system);
        display_object.add_child(&cursor_system);
        // Display_size.(x/y).floor() makes space for all lines/glyph that fit in space in
        // their full size. But we have 2 more lines/glyph: one clipped from top or left, and one
        // from bottom or right.
        const ADDITIONAL: usize = 2;
        let displayed_lines     = (window_size.y / line_height).floor() as usize + ADDITIONAL;
        let space_width         = fonts.get_render_info(font_id).get_glyph_info(' ').advance;
        let displayed_chars     = (window_size.x / space_width).floor();
        // This margin is to ensure, that after x scrolling we won't need to refresh all the lines
        // at once.
        let x_margin           = (displayed_lines as f32) / space_width;
        let max_glyphs_in_line = (displayed_chars + 2.0*x_margin).floor() as usize + ADDITIONAL;
        let indexes            = 0..displayed_lines;
        let baseline_start     = Vector2::new(0.0, 0.0);
        let glyph_lines        = indexes.map(|_| glyph_system.new_empty_line(baseline_start,line_height,max_glyphs_in_line,color)).collect();
        let assignment         =  LineRenderersAssignment::new(displayed_lines,max_glyphs_in_line,line_height);


        RenderedContent { glyph_system,cursor_system,selection_system,glyph_lines,cursors,line_height,display_object,assignment}
    }

    pub fn update_glyphs(&mut self, content:&mut TextFieldContent, fonts:&mut FontRegistry) {
        let glyph_lines           = self.glyph_lines.iter_mut().enumerate();
        let lines_with_assignment = glyph_lines.zip(self.assignment.assignment.iter());
        let lines_with_fragments  = lines_with_assignment.filter_map(|(l,opt)| opt.as_ref().map(|f|(l,f)));
        let dirty_lines           = std::mem::take(&mut content.dirty_lines);
        let dirty_glyph_lines     = std::mem::take(&mut self.assignment.dirty_renderers);
        for ((index,glyph_line),fragment) in lines_with_fragments {
            if dirty_glyph_lines.contains(&index) || dirty_lines.is_dirty(fragment.line_index) {
                let mut full_content = content.full_info(fonts);
                let mut line         = full_content.line(fragment.line_index);
                let start_x          = if fragment.chars_range.start >= line.chars().len() {
                    line.baseline_start().x
                } else {
                    line.get_char_x_position(fragment.chars_range.start)
                };
                let start_y          = line.baseline_start().y;

                let line             = content.line(fragment.line_index);
                let chars            = &line.chars()[fragment.chars_range.clone()];

                glyph_line.set_baseline_start(Vector2::new(start_x,start_y));
                glyph_line.replace_text(chars.iter().cloned(),fonts);
            }
        }
    }

    pub fn update_cursors(&mut self, cursors:&Cursors, content:&mut TextFieldContentFullInfo) {
        let cursor_system = &self.cursor_system;
        let selection_system = &self.selection_system;
        self.cursors.resize_with(cursors.cursors.len(),|| Self::new_cursor_sprites(cursor_system));
        for (sprites,cursor) in self.cursors.iter_mut().zip(cursors.cursors.iter()) {
            let position = Cursor::render_position(&cursor.position,content);
            sprites.cursor.set_position(Vector3::new(position.x,position.y,0.0));
            sprites.cursor.size().set(Vector2::new(2.0,self.line_height));

            let selection = cursor.selection_range();
            let start_position = Cursor::render_position(&selection.start,content);
            let end_position   = Cursor::render_position(&selection.end,content);
            let min            = -1e30;
            let max            = 1e30;
            if selection.start == selection.end {
                sprites.selection.clear();
            } else if selection.start.line == selection.end.line {
                let width = end_position.x - start_position.x;
                sprites.selection.resize_with(1, || selection_system.new_instance());
                sprites.selection[0].set_position(start_position);
                sprites.selection[0].size().set(Vector2::new(width,self.line_height));
            } else {
                sprites.selection.resize_with(2, || selection_system.new_instance());
                sprites.selection[0].set_position(start_position);
                sprites.selection[0].size().set(Vector2::new(max, self.line_height));
                sprites.selection[1].set_position(Vector3::new(0.0,end_position.y,0.0));
                sprites.selection[1].size().set(Vector2::new(end_position.x,self.line_height));
                let lines_between = selection.end.line - selection.start.line - 1;
                if lines_between > 0 {
                    sprites.selection.resize_with(3, || selection_system.new_instance());
                    sprites.selection[2].set_position(content.line(selection.end.line-1).baseline_start());
                    sprites.selection[2].size().set(Vector2::new(max, self.line_height * lines_between as f32));
                }
            }
        }
    }

    fn new_cursor_sprites(cursor_system:&ShapeSystem) -> CursorSprites {
        CursorSprites {
            cursor    : cursor_system.new_instance(),
            selection : Vec::new(),
        }
    }
}

impl From<&RenderedContent> for DisplayObjectData {
    fn from(rendered_content:&RenderedContent) -> Self {
        rendered_content.display_object.clone_ref()
    }
}