#![allow(missing_docs)]

use crate::prelude::*;

use crate::display::shape::text::content::TextFieldContent;
use crate::display::shape::glyph::font::{FontRegistry, FontId};

use crate::display::shape::glyph::system::GlyphSystem;
use nalgebra::{Vector4, Vector2};
use crate::display::shape::text::fragment::LineRenderersAssignment;
use crate::display::object::DisplayObjectData;


// ====================
// === RenderedLine ===
// ====================

/// Rendered Line
///
/// We don't render all lines at once in TextField; instead we keep some set of rendered lines, and
/// do quick replace of their content during scrolling.
//#[derive(Debug)]
//pub struct GlyphLine {
//    pub glyphs            : crate::display::shape::glyph::system::Line,
//    pub dirty             : bool,
//}
//
//impl GlyphLine {
//    /// Creates new instance which is not assigned to any line.
//    pub fn unassigned
//    ( system            : &mut GlyphSystem
//    , max_chars_in_line : usize
//    , height            : f32
//    , color             : Vector4<f32>
//    ) -> GlyphLine {
//        let baseline_start    = Vector2::new(0.0, 0.0);
//        let assigned_fragment = None;
//        let length            = max_chars_in_line;
//        let glyphs            = system.new_empty_line(baseline_start,height,length,color);
//        let dirty             = false;
//        GlyphLine {glyphs,assigned_fragment,dirty}
//    }
//
//
//}


// ======================
// === Rendered Lines ===
// ======================

type GlyphLine = crate::display::shape::glyph::system::Line;

#[derive(Debug)]
pub struct RenderedContent {
    pub system                        : GlyphSystem,
    pub glyph_lines                   : Vec<GlyphLine>,
    pub assignment                    : LineRenderersAssignment,
    pub line_height                   : f32,
    pub display_object                : DisplayObjectData,
}

impl RenderedContent {

    /// Create the unassigned fragments for each displayed line.
    pub fn new(window_size:Vector2<f32>, line_height:f32, color:Vector4<f32>, font_id:FontId, fonts:&mut FontRegistry) -> RenderedContent {
        let mut system     = GlyphSystem::new(font_id);
        let display_object = DisplayObjectData::new(Logger::new("RenderedContent"));
        display_object.add_child(&system);
        // Display_size.(x/y).floor() makes space for all lines/glyph that fit in space in
        // their full size. But we have 2 more lines/glyph: one clipped from top or left, and one
        // from bottom or right.
        const ADDITIONAL: usize = 2;
        let displayed_lines     = window_size.y.floor() as usize + ADDITIONAL;
        let space_width         = fonts.get_render_info(font_id).get_glyph_info(' ').advance;
        let displayed_chars     = (window_size.x/space_width).floor();
        // This margin is to ensure, that after x scrolling we won't need to refresh all the lines
        // at once.
        let x_margin           = (displayed_lines as f32) / space_width;
        let max_glyphs_in_line = (displayed_chars + 2.0*x_margin).floor() as usize + ADDITIONAL;
        let indexes            = 0..displayed_lines;
        let baseline_start     = Vector2::new(0.0, 0.0);
        let glyph_lines        = indexes.map(|_| system.new_empty_line(baseline_start,line_height,max_glyphs_in_line,color)).collect();
        let assignment         =  LineRenderersAssignment::new(displayed_lines,max_glyphs_in_line,line_height);
        RenderedContent {system,glyph_lines,line_height,display_object,assignment}
    }

    pub fn update(&mut self, content:&mut TextFieldContent, fonts:&mut FontRegistry) {
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
}

impl From<&RenderedContent> for DisplayObjectData {
    fn from(rendered_content:&RenderedContent) -> Self {
        rendered_content.display_object.clone_ref()
    }
}