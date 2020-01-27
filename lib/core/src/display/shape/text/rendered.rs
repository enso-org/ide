#![allow(missing_docs)]

use crate::prelude::*;

use crate::display::shape::text::buffer::glyph_square::Pen;
use crate::display::shape::text::buffer::glyph_square::GlyphVertexPositionBuilder;
use crate::display::shape::text::buffer::glyph_square::GlyphTextureCoordsBuilder;
use crate::display::shape::text::buffer::line::LineAttributeBuilder;
use crate::display::shape::text::content::{DirtyLines, TextFieldContent, RefreshInfo};
use crate::display::shape::text::content::line::Line;
use crate::display::shape::text::content::line::LineRef;
use crate::display::shape::glyph::font::{FontRegistry, FontId};
use crate::display::shape::glyph::font::FontRenderInfo;

use nalgebra::geometry::Point2;
use std::ops::Range;
use std::ops::RangeInclusive;
use crate::display::shape::glyph::system::GlyphSystem;
use nalgebra::{Vector4, Vector2};
use crate::display::shape::text::buffer::RenderedLines;
use crate::display::shape::text::fragment::{LineFragment, DisplayedLine};
use js_sys::Math::max;
use crate::display::object::DisplayObjectData;


// ====================
// === RenderedLine ===
// ====================

/// Rendered Line
///
/// We don't render all lines at once in TextField; instead we keep some set of rendered lines, and
/// do quick replace of their content during scrolling.
#[derive(Debug)]
pub struct GlyphLine {
    pub assigned_fragment : Option<LineFragment>,
    pub glyphs            : crate::display::shape::glyph::system::Line,
    pub dirty             : bool,
}

impl GlyphLine {
    /// Creates new instance which is not assigned to any line.
    pub fn unassigned
    ( system            : &mut GlyphSystem
    , max_chars_in_line : usize
    , height            : f32
    , color             : Vector4<f32>
    ) -> GlyphLine {
        let baseline_start    = Vector2::new(0.0, 0.0);
        let assigned_fragment = None;
        let length            = max_chars_in_line;
        let glyphs            = system.new_empty_line(baseline_start,height,length,color);
        let dirty             = false;
        GlyphLine {glyphs,assigned_fragment,dirty}
    }

    pub fn update(&mut self, content:&mut TextFieldContent, fonts:&mut FontsRegistry) {
        if let Some(fragment) = &self.assigned_fragment {
            let line           = content.line_with_char_positions(fragment.line_index,fonts);
            let baseline_start = line.start_point();
            let chars          = &line.chars()[fragment.chars_range];

            self.glyphs.set_baseline_start(baseline_start);
            self.glyphs.replace_text(chars,fonts);
        }
        self.dirty = false;
    }
}

impl DisplayedLine for GlyphLine {
    fn current_assignment(&self) -> &Option<LineFragment> {
        &self.assigned_fragment
    }

    fn assign(&mut self, fragment:LineFragment) {
        self.assigned_fragment = Some(fragment);
        self.dirty             = true;
    }
}


// ======================
// === Rendered Lines ===
// ======================


#[derive(Debug)]
pub struct RenderedContent {
    pub system                        : GlyphSystem,
    pub glyph_lines                   : Vec<GlyphLine>,
    pub line_height                   : f32,
    pub assigned_lines                : RangeInclusive<usize>,
    pub next_line_to_x_scroll_refresh : usize,
    pub display_object                : DisplayObjectData,
}

impl RenderedContent {

    /// Create the unassigned fragments for each displayed line.
    pub fn new(window_size:Vector2<f32>, line_height:f32, color:Vector4<f32>, font_id:FontId) -> BufferFragments {
        let mut system           = GlyphSystem::new(font_id);
        let display_object       = DisplayObjectData::new(Logger::new("RenderedContent"));

        // Display_size.(x/y).floor() makes space for all lines/glyph that fit in space in
        // their full size. But we have 2 more lines/glyph: one clipped from top or left, and one
        // from bottom or right.
        const ADDITIONAL: usize   = 2;
        let displayed_lines       = window_size.y.floor() as usize + ADDITIONAL;
        let space_width           = refresh.font.get_glyph_info(' ').advance;
        let displayed_chars       = (window_size.x/space_width).floor();
        // This margin is to ensure, that after x scrolling we won't need to refresh all the lines
        // at once.
        let x_margin              = (displayed_lines as f64) / space_width;
        let max_chars_in_fragment = (displayed_chars + 2.0*x_margin).floor() as usize + ADDITIONAL;
        let indexes               = 0..displayed_lines;
        let unassigned_fragments  = indexes.map(|_| GlyphLine::unassigned(&mut system,max_chars_in_fragment,line_height,color));
        RenderedContent {system,line_height,display_object,
            glyph_lines                    : unassigned_fragments.collect(),
            assigned_lines                 : 1..=0,
            next_line_to_x_scroll_refresh  : 0,
        }
    }

    pub fn update_after_text_edit(&mut self, refresh:&RefreshInfo) {
        for glyph_line in &mut self.glyph_lines {
            let assigned_line = glyph_line.assigned_fragment.map(|f| f.line_index);
            let dirty         = assigned_line.map_or(false, |l| refresh.dirty_lines.is_dirty(l));
            if dirty {
                glyph_line.update(refresh.,self.fonts);
            }
        }
    }
}
