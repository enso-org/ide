//! Information about assignment displayed glyph lines to actual fragments of text field content.
//!
//! The proper assignment requires strong unit-test coverage, therefore it was separated from
//! rendering.

use crate::prelude::*;

use crate::display::shape::text::content::TextFieldContentFullInfo;

use nalgebra::Vector2;
use std::ops::Range;
use std::ops::RangeInclusive;



/// =====================
/// === LineFragment ===
/// =====================

/// Struct describing specific one line's fragment.
#[derive(Debug)]
#[derive(Clone)]
#[allow(missing_docs)]
pub struct LineFragment {
    pub line_index  : usize,
    pub chars_range : Range<usize>,
}

impl LineFragment {
    /// Tells if rendering this line's fragment will cover the x range.
    pub fn covers_displayed_range
    ( &self
    , displayed_range : &RangeInclusive<f32>
    , content         : &mut TextFieldContentFullInfo
    ) -> bool {
        let mut line       = content.line(self.line_index);
        let front_rendered = self.chars_range.start == 0;
        let back_rendered  = self.chars_range.end == line.len();
        let x_range_start  = line.get_char_x_position(self.chars_range.start);
        let x_range_end    = line.get_char_x_range(self.chars_range.end.saturating_sub(1)).end;

        let from_left  = front_rendered || *displayed_range.start() >= x_range_start;
        let from_right = back_rendered  || *displayed_range.end()   <= x_range_end;
        from_left && from_right
    }
}



// ============================
// === GlyphLinesAssignment ===
// ============================

/// Information about assignment displayed glyph lines to actual fragments of text field content.
///
/// Here we make distinction between _lines_ which are the line of text in our field, and
/// _glyph lines_ which are sets of sprites designed to render text. So, each _glyph line_ has
/// assigned line of text which it should render. We don't render the whole text, additionally after
/// scroll we try to reassign only some of _glyph lines_ to make a minimum gpu data update.
#[derive(Debug)]
pub struct GlyphLinesAssignment {
    /// The assigned line fragments for specific _glyph line_.
    pub glyph_lines_fragments: Vec<Option<LineFragment>>,
    /// The range of currently assigned _lines_.
    pub assigned_lines: RangeInclusive<usize>,
    /// List of dirty _glyph lines_ after updating assignments. Once those will be refreshed, the
    /// this set should be cleared.
    pub dirty_glyph_lines: HashSet<usize>,
    /// Maximum displayed glyphs in _glyph line_.
    pub max_glyphs_in_line: usize,
    /// Line height in pixels.
    pub line_height: f32,
    /// The x margin of rendered glyphs in pixels.
    ///
    /// To make a horizontal scrolling faster, each _glyph line_ renders not only the visible
    /// glyphs, but also some on left and right.
    pub x_margin: f32,
    /// During x scrolling we don't immediately refresh all the lines, but pick only some which are
    /// "centered" on current scroll - the rest should still have glyphs rendered in their margin.
    /// This field keeps the id of the next _glyph line_ to do such update.
    pub next_glyph_line_to_x_scroll_update: usize,
}

impl GlyphLinesAssignment {
    /// Constructor making struct without any assignment set.
    pub fn new(glyph_lines_count:usize, max_glyphs_in_line:usize, x_margin:f32, line_height:f32)
    -> Self {
        GlyphLinesAssignment {max_glyphs_in_line,line_height,x_margin,
            glyph_lines_fragments              : (0..glyph_lines_count).map(|_| None).collect(),
            assigned_lines                     : 1..=0,
            dirty_glyph_lines                  : HashSet::new(),
            next_glyph_line_to_x_scroll_update : 0,
        }
    }

    /// A number of glyph lines.
    pub fn glyph_lines_count(&self) -> usize {
        self.glyph_lines_fragments.len()
    }
}


// === Assignment update ===

/// A helper structure for making assignment updates. It takes references to GlyphLinesAssignment
/// structure and all required data to make proper reassignments.
#[derive(Debug)]
pub struct GlyphLinesAssignmentUpdate<'a,'b,'c> {
    /// A reference to assignment structure.
    pub assignment: &'a mut GlyphLinesAssignment,
    /// A reference to TextField content.
    pub content: TextFieldContentFullInfo<'b,'c>,
    /// Current scroll offset in pixels.
    pub scroll_offset: Vector2<f32>,
    /// Current view size in pixels.
    pub view_size: Vector2<f32>,
}

impl<'a,'b,'c> GlyphLinesAssignmentUpdate<'a,'b,'c> {
    /// Reassign _glyph line_ to currently displayed fragment of line.
    pub fn reassign(&mut self, glyph_line_id:usize, line_id:usize) {
        let fragment = self.displayed_fragment(line_id);
        self.assignment.glyph_lines_fragments[glyph_line_id] = Some(fragment);
        self.assignment.dirty_glyph_lines.insert(glyph_line_id);
    }

    /// Make minimum line reassignment to cover the all displayed lines.
    pub fn update_line_assignment(&mut self) {
        let old_assignment  = self.assignment.assigned_lines.clone();
        let new_assignment  = self.new_assignment();
        let new_on_top      = *new_assignment.start().. *old_assignment.start();
        let new_on_bottom   = old_assignment.end()+1 ..=*new_assignment.end();
        let mut new_lines   = new_on_top.chain(new_on_bottom);

        for glyph_line_id in 0..self.assignment.glyph_lines_count() {
            if self.assignment.can_be_reassigned(glyph_line_id,&new_assignment) {
                if let Some(fragment) = new_lines.next() {
                    self.reassign(glyph_line_id,fragment)
                } else {
                    break;
                }
            }
        }
        self.assignment.assigned_lines = new_assignment;
    }

    /// Update some line's fragments assigned to glyph_lines after horizontal scrolling.
    pub fn update_after_x_scroll(&mut self, x_scroll:f32) {
        let updated_count = (x_scroll.abs() / self.assignment.line_height).ceil() as usize;
        let updated_count = updated_count.min(self.assignment.glyph_lines_count());
        for glyph_line_id in 0..self.assignment.glyph_lines_count() {
            if self.should_be_updated_after_x_scroll(glyph_line_id,updated_count) {
                let fragment   = self.assignment.glyph_lines_fragments[glyph_line_id].as_ref();
                let line_index = fragment.unwrap().line_index;
                self.reassign(glyph_line_id,line_index);
            }
        }
        self.assignment.increment_next_glyph_line_to_x_scroll_update(updated_count);
    }

    /// Update assigned fragments after text edit.
    ///
    /// Some new lines could be created after edit, and some lines can be longer, what should be
    /// reflected in assigned fragments.
    pub fn update_after_text_edit(&mut self) {
        let dirty_lines = std::mem::take(&mut self.content.dirty_lines);
        if self.content.dirty_lines.range.is_some() {
            self.update_line_assignment();
        }
        for i in 0..self.assignment.glyph_lines_fragments.len() {
            let assigned_fragment = &self.assignment.glyph_lines_fragments[i];
            let assigned_line     = assigned_fragment.as_ref().map(|f| f.line_index);
            match assigned_line {
                Some(line) if dirty_lines.is_dirty(line) => self.reassign(i,line),
                _                                        => {},
            }
        }
    }
}


// === Private functions ===

impl GlyphLinesAssignment {
    /// Check if given _glyph line_ could be reassigned to another line assuming some set
    /// of visible lines.
    fn can_be_reassigned(&self, glyph_line_id:usize, displayed_lines:&RangeInclusive<usize>)
    -> bool {
        match &self.glyph_lines_fragments[glyph_line_id] {
            Some(fragment) => !displayed_lines.contains(&fragment.line_index),
            None           => true
        }
    }

    /// Increment the `next_glyph_line_to_x_scroll_update` field after updating.
    fn increment_next_glyph_line_to_x_scroll_update(&mut self, updated_count:usize) {
        self.next_glyph_line_to_x_scroll_update += updated_count;
        while self.next_glyph_line_to_x_scroll_update >= self.glyph_lines_fragments.len() {
            self.next_glyph_line_to_x_scroll_update -= self.glyph_lines_fragments.len();
        }
    }
}

impl<'a,'b,'c> GlyphLinesAssignmentUpdate<'a,'b,'c> {
    /// Returns LineFragment of specific line which is currently visible.
    fn displayed_fragment(&mut self, line_id:usize) -> LineFragment {
        let mut line             = self.content.line(line_id);
        let max_index            = line.len().saturating_sub(self.assignment.max_glyphs_in_line);
        let displayed_from_x     = self.scroll_offset.x - self.assignment.x_margin;
        let first_displayed      = line.find_char_at_x_position(displayed_from_x);
        let line_front_displayed = self.scroll_offset.x <= 0.0;

        let start = match first_displayed {
            Some(index)                  => index.min(max_index),
            None if line_front_displayed => 0,
            None                         => max_index
        };
        let end = (start + self.assignment.max_glyphs_in_line).min(line.len());
        LineFragment {
            line_index: line_id,
            chars_range: start..end,
        }
    }

    /// Returns new required line assignment range, which makes minimal change from current
    /// assignment state.
    fn new_assignment(&self) -> RangeInclusive<usize> {
        let assigned_lines        = &self.assignment.assigned_lines;
        let visible_lines         = self.visible_lines_range();
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

    /// Returns range of currently visible lines.
    fn visible_lines_range(&self) -> RangeInclusive<usize> {
        let line_height              = self.assignment.line_height;
        let lines_count              = self.content.lines.len();
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

    /// Check if given _glyph line_ should be updated after x scroll.
    fn should_be_updated_after_x_scroll(&mut self, glyph_line_id:usize, updated_count:usize)
    -> bool {
        if let Some(fragment) = &self.assignment.glyph_lines_fragments[glyph_line_id] {
            let content           = &mut self.content;
            let lines_count       = self.assignment.glyph_lines_count();
            let first             = self.assignment.next_glyph_line_to_x_scroll_update;
            let last              = first + updated_count - 1;
            let last_overlap      = last as isize - lines_count as isize;
            let displayed_start   = self.scroll_offset.x;
            let displayed_end     = self.scroll_offset.x + self.view_size.x;
            let displayed_range   = displayed_start..=displayed_end;

            let not_covering      = !fragment.covers_displayed_range(&displayed_range,content);
            let to_update         = (first..=last).contains(&glyph_line_id);
            let to_update_overlap = (glyph_line_id as isize) < last_overlap;
            not_covering || to_update || to_update_overlap
        } else {
            false
        }
    }
}



//#[cfg(test)]
//mod tests {
//    use super::*;
//
//
//    use basegl_core_msdf_sys::test_utils::TestAfterInit;
//    use std::future::Future;
//    use wasm_bindgen_test::wasm_bindgen_test;
//
//    #[test]
//    fn fragment_reassignments() {
//        let lines_range = 4..=6;
//
//        assert!( make_assigned_fragment(2)     .can_be_reassigned(&lines_range));
//        assert!(!make_assigned_fragment(4)     .can_be_reassigned(&lines_range));
//        assert!(!make_assigned_fragment(6)     .can_be_reassigned(&lines_range));
//        assert!( make_assigned_fragment(7)     .can_be_reassigned(&lines_range));
//        assert!( GlyphLine::unassigned().can_be_reassigned(&lines_range));
//    }
//
//    #[test]
//    fn rendered_fragment_updating() {
//        let line            = Line::new("AAAĘĘĘ");
//        let x_range         = 1.0..=12.8;
//        let rendered_front  = LineFragment { chars_range:0..3, x_range:x_range.clone()};
//        let rendered_middle = LineFragment { chars_range:2..5, x_range:x_range.clone()};
//        let rendered_back   = LineFragment { chars_range:2..6, x_range:x_range.clone()};
//        let not_scrolled    = 1.1..=12.7;
//        let scrolled_left   = 0.9..=12.0;
//        let scrolled_right  = 2.0..=13.0;
//
//        assert!(!rendered_middle.should_be_updated(&not_scrolled  ,&line));
//        assert!( rendered_middle.should_be_updated(&scrolled_left ,&line));
//        assert!( rendered_middle.should_be_updated(&scrolled_right,&line));
//        assert!(!rendered_front .should_be_updated(&not_scrolled  ,&line));
//        assert!(!rendered_front .should_be_updated(&scrolled_left ,&line));
//        assert!( rendered_front .should_be_updated(&scrolled_right,&line));
//        assert!(!rendered_back  .should_be_updated(&not_scrolled  ,&line));
//        assert!( rendered_back  .should_be_updated(&scrolled_left ,&line));
//        assert!(!rendered_back  .should_be_updated(&scrolled_right,&line));
//    }
//
//    #[wasm_bindgen_test(async)]
//    fn build_data_for_empty_line() -> impl Future<Output=()> {
//        TestAfterInit::schedule(|| {
//            let mut font     = FontRenderInfo::mock_font("Test font".to_string());
//            let mut line     = Line::empty();
//            let mut line_ref = LineRef {line:&mut line, line_id:0};
//            let mut builder = FragmentsDataBuilder {
//                vertex_position_data : Vec::new(),
//                texture_coords_data  : Vec::new(),
//                font                 : &mut font,
//                line_clip            : 10.0..80.0,
//                max_chars_in_fragment: 100
//            };
//
//            let result = builder.build_for_line(&mut line_ref);
//
//            let expected_data = vec![0.0; 12 * 100];
//            assert!(result.is_none());
//            assert_eq!(expected_data, builder.vertex_position_data);
//            assert_eq!(expected_data, builder.texture_coords_data);
//        })
//    }
//
//    #[wasm_bindgen_test(async)]
//    fn build_data_various_lines() -> impl Future<Output=()> {
//        TestAfterInit::schedule(|| {
//            let mut font          = FontRenderInfo::mock_font("Test font".to_string());
//            let mut a_info        = font.mock_char_info('A');
//            a_info.advance        = 1.0;
//            let mut b_info        = font.mock_char_info('B');
//            b_info.advance        = 1.5;
//            font.mock_kerning_info('A', 'A', 0.0);
//            font.mock_kerning_info('B', 'B', 0.0);
//            font.mock_kerning_info('A', 'B', 0.0);
//            font.mock_kerning_info('B', 'A', 0.0);
//            let mut shortest_line = Line::new("AB"         .to_string());
//            let mut short_line    = Line::new("ABBA"       .to_string());
//            let mut medium_line   = Line::new("ABBAAB"     .to_string());
//            let mut long_line     = Line::new("ABBAABBABBA".to_string());
//
//            let mut builder = FragmentsDataBuilder {
//                vertex_position_data : Vec::new(),
//                texture_coords_data  : Vec::new(),
//                font                 : &mut font,
//                line_clip            : 5.5..8.0,
//                max_chars_in_fragment: 3
//            };
//            let shortest_result = builder.build_for_line(&mut LineRef {line:&mut shortest_line, line_id:1}).unwrap();
//            let short_result    = builder.build_for_line(&mut LineRef {line:&mut short_line, line_id:2}).unwrap();
//            let medium_result   = builder.build_for_line(&mut LineRef {line:&mut medium_line, line_id:3}).unwrap();
//            let long_result     = builder.build_for_line(&mut LineRef {line:&mut long_line, line_id:4}).unwrap();
//
//            assert_eq!(0..2, shortest_result.chars_range);
//            assert_eq!(1..4, short_result   .chars_range);
//            assert_eq!(3..6, medium_result  .chars_range);
//            assert_eq!(4..7, long_result    .chars_range);
//
//            assert_eq!(0.0..=2.5, shortest_result.x_range);
//            assert_eq!(1.0..=5.0, short_result   .x_range);
//            assert_eq!(4.0..=7.5, medium_result  .x_range);
//            assert_eq!(5.0..=9.0, long_result    .x_range);
//
//            let vertex_glyph_data_size    = GlyphVertexPositionBuilder::OUTPUT_SIZE;
//            let tex_coord_glyph_data_size = GlyphTextureCoordsBuilder::OUTPUT_SIZE;
//            let glyphs_count              = builder.max_chars_in_fragment * 4;
//            let vertex_data_size          = vertex_glyph_data_size * glyphs_count;
//            let tex_coord_data_size       = tex_coord_glyph_data_size * glyphs_count;
//            assert_eq!(vertex_data_size   , builder.vertex_position_data.len());
//            assert_eq!([0.0, -2.0], builder.vertex_position_data[0..2]);
//            assert_eq!(tex_coord_data_size, builder.texture_coords_data.len());
//        })
//    }
//
//    #[wasm_bindgen_test(async)]
//    fn build_data_with_non_ascii() -> impl Future<Output=()> {
//        TestAfterInit::schedule(|| {
//            let mut font     = FontRenderInfo::mock_font("Test font".to_string());
//            let mut a_info   = font.mock_char_info('Ą');
//            a_info.advance   = 1.0;
//            let mut b_info   = font.mock_char_info('B');
//            b_info.advance   = 1.5;
//            font.mock_kerning_info('Ą', 'B', 0.0);
//            let mut line     = Line::new("ĄB".to_string());
//            let mut line_ref = LineRef {line:&mut line, line_id:0};
//
//            let mut builder = FragmentsDataBuilder {
//                vertex_position_data : Vec::new(),
//                texture_coords_data  : Vec::new(),
//                font                 : &mut font,
//                line_clip            : 0.0..10.0,
//                max_chars_in_fragment: 3
//            };
//            let result = builder.build_for_line(&mut line_ref).unwrap();
//
//            assert_eq!(0..2, result.chars_range);
//        })
//    }
//
//    #[test]
//    fn fragments_reassign() {
//        let assigned_lines   = 4..=6;
//        let new_assignment_1 = 2..=5;
//        let new_assignment_2 = 5..=8;
//        let mut fragments = BufferFragments {
//            assigned_indices: assigned_lines.clone(),
//            rendered_lines: assigned_lines.map(make_assigned_fragment).collect(),
//            next_to_refresh : NextGlyphLineToRefreshAfterXScrolling::new(4)
//        };
//        fragments.rendered_lines.push(GlyphLine::unassigned());
//
//        fragments.reassign_fragments(new_assignment_1.clone());
//        let expected_assignments_1 = vec![4,5,2,3];
//        let assignments_1_iter     = fragments.rendered_lines.iter().map(|f| f.assigned_index.unwrap());
//        let assignments_1          = assignments_1_iter.collect::<Vec<usize>>();
//        assert_eq!(new_assignment_1      , fragments.assigned_indices);
//        assert_eq!(expected_assignments_1, assignments_1);
//
//        fragments.reassign_fragments(new_assignment_2.clone());
//        let expected_assignments_2 = vec![6,5,7,8];
//        let assignments_2_iter     = fragments.rendered_lines.iter().map(|f| f.assigned_index.unwrap());
//        let assignments_2          = assignments_2_iter.collect::<Vec<usize>>();
//        assert_eq!(new_assignment_2      , fragments.assigned_indices);
//        assert_eq!(expected_assignments_2, assignments_2);
//    }
//
//    #[test]
//    fn marking_dirty_after_x_scrolling() {
//        let make_rendered      = |x_range| LineFragment { chars_range:1..3, x_range};
//        let displayed_range    = 4.0..=6.0;
//        let rendered_not_dirty = make_rendered(3.0..=7.0);
//        let rendered_dirty     = make_rendered(5.0..=7.0);
//        let displayed_lines    = 5;
//        let lines_iter         = (0..displayed_lines).map(|_| Line::new("AA".to_string()));
//        let lines              = lines_iter.collect::<Vec<Line>>();
//
//        let mut fragments      = BufferFragments::new(displayed_lines);
//        for (i,fragment) in fragments.rendered_lines.iter_mut().take(4).enumerate() {
//            fragment.assigned_index = Some(i);
//        }
//        fragments.rendered_lines[0].glyphs = Some(rendered_dirty);
//        fragments.rendered_lines[2].glyphs = Some(rendered_not_dirty.clone());
//        fragments.rendered_lines[3].glyphs = Some(rendered_not_dirty);
//        fragments.rendered_lines[3].dirty    = true;
//
//        fragments.mark_dirty_after_x_scrolling(0.0,displayed_range,&lines);
//
//        let dirties          = fragments.rendered_lines.iter().map(|f| f.dirty).collect::<Vec<bool>>();
//        let expected_dirties = vec![true, true, false, true, false];
//        assert_eq!(expected_dirties, dirties);
//    }
//
//    fn make_assigned_fragment(line:usize) -> GlyphLine {
//        GlyphLine {
//            assigned_index: Some(line),
//            glyphs: None,
//            dirty         : true,
//        }
//    }
//}
