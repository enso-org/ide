use crate::prelude::*;

use std::ops::{Range, RangeInclusive};
use crate::display::shape::text::content::{TextFieldContent, DirtyLines};
use crate::display::shape::glyph::font::FontRegistry;
use nalgebra::Vector2;

/// ========================
/// === RenderedFragment ===
/// ========================

/// Information what is currently rendered on screen for some specific _buffer fragment_.
#[derive(Debug)]
#[derive(Clone)]
pub struct LineFragment {
    pub line_index  : usize,
    pub chars_range : Range<usize>,
}

impl LineFragment {

    /// Tells if fragment needs to be updated because currently rendered content does not covers
    /// all displayed part of line.
    pub fn should_be_updated(&self, displayed_range:&RangeInclusive<f32>, content:&mut TextFieldContent) -> bool {
        let line           = content.line(self.line_index);
        let front_rendered = self.chars_range.start == 0;
        let back_rendered  = self.chars_range.end == line.len();
        let x_range_start  = *self.x_range.start();
        let x_range_end    = *self.x_range.end();

        let has_on_left    = !front_rendered && *displayed_range.start() < x_range_start;
        let has_on_right   = !back_rendered  && *displayed_range.end()   > x_range_end;
        has_on_left || has_on_right
    }
}

pub trait DisplayedLine {
    fn current_assignment(&self) -> &Option<LineFragment>;
    fn assign(&mut self, fragment:LineFragment);

    /// Basing of list of current displayed lines, this function tells if the fragment can be
    /// assigned to another line.
    fn can_be_reassigned(&self, displayed_lines:&RangeInclusive<usize>) -> bool {
        match self.current_assignment() {
            Some(fragment) => !displayed_lines.contains(&fragment.line_index),
            None           => true
        }
    }
}


pub struct DisplayedLinesUpdate<'a,'b,'c,'d,'e,Line> {
    pub content                      : &'a mut TextFieldContent,
    pub displayed_lines              : &'b mut [Line],
    pub fonts                        : &'c mut FontRegistry,
    pub assigned_lines               : &'d mut RangeInclusive<usize>,
    pub next_line_to_x_scroll_update : &'e mut usize,
    pub scroll_offset                : Vector2<f32>,
    pub view_size                    : Vector2<f32>,
    pub line_height                  : f32,
    pub max_glyphs_in_line           : usize,
}

impl<'a,'b,'c,'d,'e,Line:DisplayedLine> DisplayedLinesUpdate<'a,'b,'c,'d,'e,Line> {
    /// Make minimum fragments reassignment to cover the all displayed lines.
    pub fn reassign_after_x_scroll(&mut self) {
        let current_assignment = self.assigned_lines;
        let new_assignment     = self.new_assignment();
        let new_on_top         = *new_assignment.start()   .. *current_assignment.start();
        let new_on_bottom      = current_assignment.end()+1..=*new_assignment.end();
        let new_lines          = new_on_top.chain(new_on_bottom);
        let displayed_lines    = self.displayed_lines.iter_mut();
        let free_lines         = displayed_lines.filter(|f| f.can_be_reassigned(&new_assignment));
        let reassignments      = new_lines.zip(free_glyph_lines);

        for (line_id,displayed_line) in reassignments {
            displayed_line.assign(self.displayed_fragment(line_id));
        }
        self.rendered_content.assigned_lines = new_assignment;
    }

    /// Returns new assignment for displayed lines, which makes minimal required reassignments of
    /// currently rendered GlyphLines.
    pub fn new_assignment(&self) -> RangeInclusive<usize> {
        let assigned_lines        = &self.assigned_lines;
        let visible_lines         = self.visible_lines_range(self.displayed_lines.len());
        let lines_count           = |r:&RangeInclusive<usize>| r.end() + 1 - r.start();
        let assigned_lines_count  = lines_count(assigned_lines);
        let displayed_lines_count = lines_count(&visible_lines);
        let hidden_lines_to_keep  = assigned_lines_count.saturating_sub(displayed_lines_count);
        if assigned_lines.start() < visible_lines.start() {
            let new_start = visible_lines.start() - hidden_lines_to_keep;
            new_start..=*visible_lines.end()
        } else if assigned_lines.end() > visible_lines.end() {
            let new_end = visible_lines.end() + hidden_lines_to_keep;
            *visible_lines.start()..=new_end
        } else {
            visible_lines
        }
    }

    fn visible_lines_range(&self, lines_count:usize) -> RangeInclusive<usize> {
        let line_height              = self.line_height;
        let lines_count              = self.content.len();
        let top                      = self.scroll_offset.y;
        let bottom                   = self.scroll_offset.y - self.view_size.y;
        let top_line_clipped         = Self::line_at_y_position(top,line_height,lines_count);
        let bottom_line_clipped      = Self::line_at_y_position(bottom,line_height,lines_count);
        let first_line_index         = top_line_clipped.unwrap_or(0);
        let last_line_index          = bottom_line_clipped.unwrap_or(lines_count-1);
        first_line_index..=last_line_index
    }

    fn line_at_y_position(y:f32, line_height:f32, lines_count:usize) -> Option<usize> {
        let index    = -(y / line_height).ceil();
        let is_valid = index >= 0.0 && index < lines_count as f32;
        is_valid.and_option_from(|| Some(index as usize))
    }

    fn displayed_fragment(&self, line_id:usize) -> LineFragment {
        let font                 = self.fonts.get_render_info(self.content.font);
        let line                 = self.content.line(line_id);
        let max_index            = line.len().saturating_sub(self.max_glyphs_in_line);
        let first_displayed      = line.find_char_at_x_position(*self.scroll_offset.x,font);
        let line_front_displayed = *displayed_x.start() <= 0.0;

        let start = match first_displayed {
            Some(index)                  => index.min(max_index),
            None if line_front_displayed => 0,
            None                         => max_index
        };
        let end = (start + self.max_glyphs_in_line).min(line.len());
        LineFragment {
            line_index: line_id,
            chars_range: start..end,
        }
    }

    pub fn update_after_x_scroll(&mut self, x_scroll:f32) {
        let updated_count = (x_scroll.abs() / self.line_height).ceil() as usize;
        for i in 0..updated_count {
            let line          = &mut self.displayed_lines[self.next_line_to_x_scroll_update];
            let assigned_line = line.current_assignment().map(|f| f.line_id);
            if let Some(line) = assigned_line {
                line.assign(self.displayed_fragment(line));
            }
            *self.next_line_to_x_scroll_update += 1;
            if self.next_line_to_x_scroll_update >= self.displayed_lines.len() {
                *self.next_line_to_x_scroll_update = 0;
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    use crate::display::shape::text::buffer::glyph_square::GlyphAttributeBuilder;
    use crate::display::shape::text::buffer::glyph_square::GlyphVertexPositionBuilder;

    use basegl_core_msdf_sys::test_utils::TestAfterInit;
    use std::future::Future;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    fn fragment_reassignments() {
        let lines_range = 4..=6;

        assert!( make_assigned_fragment(2)     .can_be_reassigned(&lines_range));
        assert!(!make_assigned_fragment(4)     .can_be_reassigned(&lines_range));
        assert!(!make_assigned_fragment(6)     .can_be_reassigned(&lines_range));
        assert!( make_assigned_fragment(7)     .can_be_reassigned(&lines_range));
        assert!( GlyphLine::unassigned().can_be_reassigned(&lines_range));
    }

    #[test]
    fn rendered_fragment_updating() {
        let line            = Line::new("AAAĘĘĘ");
        let x_range         = 1.0..=12.8;
        let rendered_front  = LineFragment { chars_range:0..3, x_range:x_range.clone()};
        let rendered_middle = LineFragment { chars_range:2..5, x_range:x_range.clone()};
        let rendered_back   = LineFragment { chars_range:2..6, x_range:x_range.clone()};
        let not_scrolled    = 1.1..=12.7;
        let scrolled_left   = 0.9..=12.0;
        let scrolled_right  = 2.0..=13.0;

        assert!(!rendered_middle.should_be_updated(&not_scrolled  ,&line));
        assert!( rendered_middle.should_be_updated(&scrolled_left ,&line));
        assert!( rendered_middle.should_be_updated(&scrolled_right,&line));
        assert!(!rendered_front .should_be_updated(&not_scrolled  ,&line));
        assert!(!rendered_front .should_be_updated(&scrolled_left ,&line));
        assert!( rendered_front .should_be_updated(&scrolled_right,&line));
        assert!(!rendered_back  .should_be_updated(&not_scrolled  ,&line));
        assert!( rendered_back  .should_be_updated(&scrolled_left ,&line));
        assert!(!rendered_back  .should_be_updated(&scrolled_right,&line));
    }

    #[wasm_bindgen_test(async)]
    fn build_data_for_empty_line() -> impl Future<Output=()> {
        TestAfterInit::schedule(|| {
            let mut font     = FontRenderInfo::mock_font("Test font".to_string());
            let mut line     = Line::empty();
            let mut line_ref = LineRef {line:&mut line, line_id:0};
            let mut builder = FragmentsDataBuilder {
                vertex_position_data : Vec::new(),
                texture_coords_data  : Vec::new(),
                font                 : &mut font,
                line_clip            : 10.0..80.0,
                max_chars_in_fragment: 100
            };

            let result = builder.build_for_line(&mut line_ref);

            let expected_data = vec![0.0; 12 * 100];
            assert!(result.is_none());
            assert_eq!(expected_data, builder.vertex_position_data);
            assert_eq!(expected_data, builder.texture_coords_data);
        })
    }

    #[wasm_bindgen_test(async)]
    fn build_data_various_lines() -> impl Future<Output=()> {
        TestAfterInit::schedule(|| {
            let mut font          = FontRenderInfo::mock_font("Test font".to_string());
            let mut a_info        = font.mock_char_info('A');
            a_info.advance        = 1.0;
            let mut b_info        = font.mock_char_info('B');
            b_info.advance        = 1.5;
            font.mock_kerning_info('A', 'A', 0.0);
            font.mock_kerning_info('B', 'B', 0.0);
            font.mock_kerning_info('A', 'B', 0.0);
            font.mock_kerning_info('B', 'A', 0.0);
            let mut shortest_line = Line::new("AB"         .to_string());
            let mut short_line    = Line::new("ABBA"       .to_string());
            let mut medium_line   = Line::new("ABBAAB"     .to_string());
            let mut long_line     = Line::new("ABBAABBABBA".to_string());

            let mut builder = FragmentsDataBuilder {
                vertex_position_data : Vec::new(),
                texture_coords_data  : Vec::new(),
                font                 : &mut font,
                line_clip            : 5.5..8.0,
                max_chars_in_fragment: 3
            };
            let shortest_result = builder.build_for_line(&mut LineRef {line:&mut shortest_line, line_id:1}).unwrap();
            let short_result    = builder.build_for_line(&mut LineRef {line:&mut short_line, line_id:2}).unwrap();
            let medium_result   = builder.build_for_line(&mut LineRef {line:&mut medium_line, line_id:3}).unwrap();
            let long_result     = builder.build_for_line(&mut LineRef {line:&mut long_line, line_id:4}).unwrap();

            assert_eq!(0..2, shortest_result.chars_range);
            assert_eq!(1..4, short_result   .chars_range);
            assert_eq!(3..6, medium_result  .chars_range);
            assert_eq!(4..7, long_result    .chars_range);

            assert_eq!(0.0..=2.5, shortest_result.x_range);
            assert_eq!(1.0..=5.0, short_result   .x_range);
            assert_eq!(4.0..=7.5, medium_result  .x_range);
            assert_eq!(5.0..=9.0, long_result    .x_range);

            let vertex_glyph_data_size    = GlyphVertexPositionBuilder::OUTPUT_SIZE;
            let tex_coord_glyph_data_size = GlyphTextureCoordsBuilder::OUTPUT_SIZE;
            let glyphs_count              = builder.max_chars_in_fragment * 4;
            let vertex_data_size          = vertex_glyph_data_size * glyphs_count;
            let tex_coord_data_size       = tex_coord_glyph_data_size * glyphs_count;
            assert_eq!(vertex_data_size   , builder.vertex_position_data.len());
            assert_eq!([0.0, -2.0], builder.vertex_position_data[0..2]);
            assert_eq!(tex_coord_data_size, builder.texture_coords_data.len());
        })
    }

    #[wasm_bindgen_test(async)]
    fn build_data_with_non_ascii() -> impl Future<Output=()> {
        TestAfterInit::schedule(|| {
            let mut font     = FontRenderInfo::mock_font("Test font".to_string());
            let mut a_info   = font.mock_char_info('Ą');
            a_info.advance   = 1.0;
            let mut b_info   = font.mock_char_info('B');
            b_info.advance   = 1.5;
            font.mock_kerning_info('Ą', 'B', 0.0);
            let mut line     = Line::new("ĄB".to_string());
            let mut line_ref = LineRef {line:&mut line, line_id:0};

            let mut builder = FragmentsDataBuilder {
                vertex_position_data : Vec::new(),
                texture_coords_data  : Vec::new(),
                font                 : &mut font,
                line_clip            : 0.0..10.0,
                max_chars_in_fragment: 3
            };
            let result = builder.build_for_line(&mut line_ref).unwrap();

            assert_eq!(0..2, result.chars_range);
        })
    }

    #[test]
    fn fragments_reassign() {
        let assigned_lines   = 4..=6;
        let new_assignment_1 = 2..=5;
        let new_assignment_2 = 5..=8;
        let mut fragments = BufferFragments {
            assigned_indices: assigned_lines.clone(),
            rendered_lines: assigned_lines.map(make_assigned_fragment).collect(),
            next_to_refresh : NextGlyphLineToRefreshAfterXScrolling::new(4)
        };
        fragments.rendered_lines.push(GlyphLine::unassigned());

        fragments.reassign_fragments(new_assignment_1.clone());
        let expected_assignments_1 = vec![4,5,2,3];
        let assignments_1_iter     = fragments.rendered_lines.iter().map(|f| f.assigned_index.unwrap());
        let assignments_1          = assignments_1_iter.collect::<Vec<usize>>();
        assert_eq!(new_assignment_1      , fragments.assigned_indices);
        assert_eq!(expected_assignments_1, assignments_1);

        fragments.reassign_fragments(new_assignment_2.clone());
        let expected_assignments_2 = vec![6,5,7,8];
        let assignments_2_iter     = fragments.rendered_lines.iter().map(|f| f.assigned_index.unwrap());
        let assignments_2          = assignments_2_iter.collect::<Vec<usize>>();
        assert_eq!(new_assignment_2      , fragments.assigned_indices);
        assert_eq!(expected_assignments_2, assignments_2);
    }

    #[test]
    fn marking_dirty_after_x_scrolling() {
        let make_rendered      = |x_range| LineFragment { chars_range:1..3, x_range};
        let displayed_range    = 4.0..=6.0;
        let rendered_not_dirty = make_rendered(3.0..=7.0);
        let rendered_dirty     = make_rendered(5.0..=7.0);
        let displayed_lines    = 5;
        let lines_iter         = (0..displayed_lines).map(|_| Line::new("AA".to_string()));
        let lines              = lines_iter.collect::<Vec<Line>>();

        let mut fragments      = BufferFragments::new(displayed_lines);
        for (i,fragment) in fragments.rendered_lines.iter_mut().take(4).enumerate() {
            fragment.assigned_index = Some(i);
        }
        fragments.rendered_lines[0].glyphs = Some(rendered_dirty);
        fragments.rendered_lines[2].glyphs = Some(rendered_not_dirty.clone());
        fragments.rendered_lines[3].glyphs = Some(rendered_not_dirty);
        fragments.rendered_lines[3].dirty    = true;

        fragments.mark_dirty_after_x_scrolling(0.0,displayed_range,&lines);

        let dirties          = fragments.rendered_lines.iter().map(|f| f.dirty).collect::<Vec<bool>>();
        let expected_dirties = vec![true, true, false, true, false];
        assert_eq!(expected_dirties, dirties);
    }

    fn make_assigned_fragment(line:usize) -> GlyphLine {
        GlyphLine {
            assigned_index: Some(line),
            glyphs: None,
            dirty         : true,
        }
    }
}
