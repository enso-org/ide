//! Ensogl text rendering implementation.

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

pub mod glyph;

/// Commonly used types and functions.
pub mod prelude {
    pub use ensogl::prelude::*;
}

pub use ensogl::display;

//use prelude::*;
//
//
use xi_rope::spans::Spans;
use xi_rope::spans::SpansBuilder;
use xi_rope::breaks::{BreakBuilder, Breaks, BreaksInfo, BreaksMetric};
use xi_rope::{Cursor, Interval, LinesMetric, Rope, RopeDelta, RopeInfo};
//use xi_rope::LinesMetric;
//use xi_rope::rope::BaseMetric;
//use xi_rope::tree::*;
//
//
//
//
//
//
//pub struct Line {
//    text  : Rope,
//    index : usize,
//}

use std::cmp::max;
use std::cmp::min;



pub fn main() {
    let mut txt = Buffer::from("Test text!");
    txt.set_color(1..3,color::Rgba::new(1.0,0.0,0.0,1.0));

//    let foo = txt.color.iter().collect_vec();
    let foo = txt.color.subseq(2..5);
    let foo = foo.iter().collect_vec();
    println!("{:#?}",foo);
}



#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct BufferId(pub usize);

pub struct BufferMap {
    map : BTreeMap<BufferId,Buffer>
}



// ==============
// === Buffer ===
// ==============

#[derive(Debug,Clone,Default)]
pub struct Buffer {
    /// The contents of the buffer.
    pub rope: Rope,

    // /// The CRDT engine, which tracks edit history and manages concurrent edits.
    // engine: Engine,

    pub color : Spans<color::Rgba>,
}

impl Buffer {
    pub fn new() -> Self {
        default()
    }

    pub fn set_color(&mut self, interval:impl Into<Interval>, color:impl Into<color::Rgba>) {
        let interval = interval.into();
        let color    = color.into();

        let mut sb = SpansBuilder::new(interval.end());
        sb.add_span(interval,color);

        self.color.edit(interval,sb.build());
    }
}


// === Conversions ===

impl From<Rope> for Buffer {
    fn from(rope:Rope) -> Self {
        Self {rope,..default()}
    }
}

impl From<&Rope> for Buffer {
    fn from(rope:&Rope) -> Self {
        let rope = rope.clone();
        Self {rope,..default()}
    }
}

impl From<&str> for Buffer {
    fn from(s:&str) -> Self {
        Rope::from(s).into()
    }
}

impl From<String> for Buffer {
    fn from(s:String) -> Self {
        Rope::from(s).into()
    }
}

impl From<&String> for Buffer {
    fn from(s:&String) -> Self {
        Rope::from(s).into()
    }
}

impl From<&&String> for Buffer {
    fn from(s:&&String) -> Self {
        (*s).into()
    }
}

impl From<&&str> for Buffer {
    fn from(s:&&str) -> Self {
        (*s).into()
    }
}



// ============
// === View ===
// ============

pub struct View {
    buffer_id : BufferId,
    /// vertical scroll position
    first_line: usize,
    /// height of visible portion
    height: usize,
    selection: Selection,
    /// New offset to be scrolled into position after an edit.
    scroll_to: Option<usize>,
}


impl LineOffset for View {
    fn offset_of_line(&self, text:&Rope, line:usize) -> usize {
        let line = line.min(text.measure::<LinesMetric>() + 1);
        text.offset_of_line(line)
    }

    fn line_of_offset(&self, text:&Rope, offset:usize) -> usize {
        text.line_of_offset(offset)
    }
}


impl View {
    /// If `modify` is `true`, the selections are modified, otherwise the results
    /// of individual region movements become carets.
    pub fn do_move(&mut self, text: &Rope, movement: Movement, modify: bool) {
        // self.drag_state = None;
        let new_sel =
            selection_movement(movement, &self.selection, self, self.scroll_height(), text, modify);
        self.set_selection(text, new_sel);
    }

    pub fn scroll_height(&self) -> usize {
        self.height
    }

    /// Returns the regions of the current selection.
    pub fn sel_regions(&self) -> &[SelRegion] {
        &self.selection
    }

    /// Set the selection to a new value.
    pub fn set_selection<S: Into<Selection>>(&mut self, text: &Rope, sel: S) {
        self.set_selection_raw(text, sel.into());
        self.scroll_to_cursor(text);
    }

    /// Sets the selection to a new value, invalidating the line cache as needed.
    /// This function does not perform any scrolling.
    fn set_selection_raw(&mut self, text: &Rope, sel: Selection) {
        self.invalidate_selection(text);
        self.selection = sel;
        self.invalidate_selection(text);
    }

    fn scroll_to_cursor(&mut self, text: &Rope) {
        let end = self.sel_regions().last().unwrap().end;
        let line = self.line_of_offset(text, end);
        if line < self.first_line {
            self.first_line = line;
        } else if self.first_line + self.height <= line {
            self.first_line = line - (self.height - 1);
        }
        // We somewhat arbitrarily choose the last region for setting the old-style
        // selection state, and for scrolling it into view if needed. This choice can
        // likely be improved.
        self.scroll_to = Some(end);
    }

    /// Invalidate the current selection. Note that we could be even more
    /// fine-grained in the case of multiple cursors, but we also want this
    /// method to be fast even when the selection is large.
    fn invalidate_selection(&mut self, text: &Rope) {
//        // TODO: refine for upstream (caret appears on prev line)
//        let first_line = self.line_of_offset(text, self.selection.first().unwrap().min());
//        let last_line = self.line_of_offset(text, self.selection.last().unwrap().max()) + 1;
//        let all_caret = self.selection.iter().all(|region| region.is_caret());
//        let invalid = if all_caret {
//            line_cache_shadow::CURSOR_VALID
//        } else {
//            line_cache_shadow::CURSOR_VALID | line_cache_shadow::STYLES_VALID
//        };
//        self.lc_shadow.partial_invalidate(first_line, last_line, invalid);
    }
}


/// Based on the current selection position this will return the cursor position, the current line, and the
/// total number of lines of the file.
fn selection_position(
    r: SelRegion,
    lo: &dyn LineOffset,
    text: &Rope,
    move_up: bool,
    modify: bool,
) -> (HorizPos, usize) {
    // The active point of the selection
    let active = if modify {
        r.end
    } else if move_up {
        r.min()
    } else {
        r.max()
    };
    let col = if let Some(col) = r.horiz { col } else { lo.offset_to_line_col(text, active).1 };
    let line = lo.line_of_offset(text, active);

    (col, line)
}


/// Compute movement based on vertical motion by the given number of lines.
///
/// Note: in non-exceptional cases, this function preserves the `horiz`
/// field of the selection region.
fn vertical_motion(
    r: SelRegion,
    lo: &dyn LineOffset,
    text: &Rope,
    line_delta: isize,
    modify: bool,
) -> (usize, Option<HorizPos>) {
    let (col, line) = selection_position(r, lo, text, line_delta < 0, modify);
    let n_lines = lo.line_of_offset(text, text.len());

    // This code is quite careful to avoid integer overflow.
    // TODO: write tests to verify
    if line_delta < 0 && (-line_delta as usize) > line {
        return (0, Some(col));
    }
    let line = if line_delta < 0 {
        line - (-line_delta as usize)
    } else {
        line.saturating_add(line_delta as usize)
    };
    if line > n_lines {
        return (text.len(), Some(col));
    }
    let new_offset = lo.line_col_to_offset(text, line, col);
    (new_offset, Some(col))
}

/// Compute movement based on vertical motion by the given number of lines skipping
/// any line that is shorter than the current cursor position.
fn vertical_motion_exact_pos(
    r: SelRegion,
    lo: &dyn LineOffset,
    text: &Rope,
    move_up: bool,
    modify: bool,
) -> (usize, Option<HorizPos>) {
    let (col, init_line) = selection_position(r, lo, text, move_up, modify);
    let n_lines = lo.line_of_offset(text, text.len());

    let mut line_length =
        lo.offset_of_line(text, init_line.saturating_add(1)) - lo.offset_of_line(text, init_line);
    if move_up && init_line == 0 {
        return (lo.line_col_to_offset(text, init_line, col), Some(col));
    }
    let mut line = if move_up { init_line - 1 } else { init_line.saturating_add(1) };

    // If the active columns is longer than the current line, use the current line length.
    let col = if line_length < col { line_length - 1 } else { col };

    loop {
        line_length = lo.offset_of_line(text, line + 1) - lo.offset_of_line(text, line);

        // If the line is longer than the current cursor position, break.
        // We use > instead of >= because line_length includes newline.
        if line_length > col {
            break;
        }

        // If you are trying to add a selection past the end of the file or before the first line, return original selection
        if line >= n_lines || (line == 0 && move_up) {
            line = init_line;
            break;
        }

        line = if move_up { line - 1 } else { line.saturating_add(1) };
    }

    (lo.line_col_to_offset(text, line, col), Some(col))
}




/// Compute the result of movement on one selection region.
///
/// # Arguments
///
/// * `height` - viewport height
pub fn region_movement(
    m: Movement,
    r: SelRegion,
    lo: &dyn LineOffset,
    height: usize,
    text: &Rope,
    modify: bool,
) -> SelRegion {
    let (offset, horiz) = match m {
        Movement::Left => {
            if r.is_caret() || modify {
                if let Some(offset) = text.prev_grapheme_offset(r.end) {
                    (offset, None)
                } else {
                    (0, r.horiz)
                }
            } else {
                (r.min(), None)
            }
        }
        Movement::Right => {
            if r.is_caret() || modify {
                if let Some(offset) = text.next_grapheme_offset(r.end) {
                    (offset, None)
                } else {
                    (r.end, r.horiz)
                }
            } else {
                (r.max(), None)
            }
        }
//        Movement::LeftWord => {
//            let mut word_cursor = WordCursor::new(text, r.end);
//            let offset = word_cursor.prev_boundary().unwrap_or(0);
//            (offset, None)
//        }
//        Movement::RightWord => {
//            let mut word_cursor = WordCursor::new(text, r.end);
//            let offset = word_cursor.next_boundary().unwrap_or_else(|| text.len());
//            (offset, None)
//        }
        Movement::LeftOfLine => {
            let line = lo.line_of_offset(text, r.end);
            let offset = lo.offset_of_line(text, line);
            (offset, None)
        }
        Movement::RightOfLine => {
            let line = lo.line_of_offset(text, r.end);
            let mut offset = text.len();

            // calculate end of line
            let next_line_offset = lo.offset_of_line(text, line + 1);
            if line < lo.line_of_offset(text, offset) {
                if let Some(prev) = text.prev_grapheme_offset(next_line_offset) {
                    offset = prev;
                }
            }
            (offset, None)
        }
        Movement::Up => vertical_motion(r, lo, text, -1, modify),
        Movement::Down => vertical_motion(r, lo, text, 1, modify),
        Movement::UpExactPosition => vertical_motion_exact_pos(r, lo, text, true, modify),
        Movement::DownExactPosition => vertical_motion_exact_pos(r, lo, text, false, modify),
        Movement::StartOfParagraph => {
            // Note: TextEdit would start at modify ? r.end : r.min()
            let mut cursor = Cursor::new(&text, r.end);
            let offset = cursor.prev::<LinesMetric>().unwrap_or(0);
            (offset, None)
        }
        Movement::EndOfParagraph => {
            // Note: TextEdit would start at modify ? r.end : r.max()
            let mut offset = r.end;
            let mut cursor = Cursor::new(&text, offset);
            if let Some(next_para_offset) = cursor.next::<LinesMetric>() {
                if cursor.is_boundary::<LinesMetric>() {
                    if let Some(eol) = text.prev_grapheme_offset(next_para_offset) {
                        offset = eol;
                    }
                } else if cursor.pos() == text.len() {
                    offset = text.len();
                }
                (offset, None)
            } else {
                //in this case we are already on a last line so just moving to EOL
                (text.len(), None)
            }
        }
        Movement::EndOfParagraphKill => {
            // Note: TextEdit would start at modify ? r.end : r.max()
            let mut offset = r.end;
            let mut cursor = Cursor::new(&text, offset);
            if let Some(next_para_offset) = cursor.next::<LinesMetric>() {
                offset = next_para_offset;
                if cursor.is_boundary::<LinesMetric>() {
                    if let Some(eol) = text.prev_grapheme_offset(next_para_offset) {
                        if eol != r.end {
                            offset = eol;
                        }
                    }
                }
            }
            (offset, None)
        }
        Movement::UpPage => vertical_motion(r, lo, text, -scroll_height(height), modify),
        Movement::DownPage => vertical_motion(r, lo, text, scroll_height(height), modify),
        Movement::StartOfDocument => (0, None),
        Movement::EndOfDocument => (text.len(), None),
    };
    SelRegion::new(if modify { r.start } else { offset }, offset).with_horiz(horiz)
}

/// When paging through a file, the number of lines from the previous page
/// that will also be visible in the next.
const SCROLL_OVERLAP: isize = 2;

/// Computes the actual desired amount of scrolling (generally slightly
/// less than the height of the viewport, to allow overlap).
fn scroll_height(height: usize) -> isize {
    max(height as isize - SCROLL_OVERLAP, 1)
}

/// Compute a new selection by applying a movement to an existing selection.
///
/// In a multi-region selection, this function applies the movement to each
/// region in the selection, and returns the union of the results.
///
/// If `modify` is `true`, the selections are modified, otherwise the results
/// of individual region movements become carets.
///
/// # Arguments
///
/// * `height` - viewport height
pub fn selection_movement(
    m: Movement,
    s: &Selection,
    lo: &dyn LineOffset,
    height: usize,
    text: &Rope,
    modify: bool,
) -> Selection {
    let mut result = Selection::new();
    for &r in s.iter() {
        let new_region = region_movement(m, r, lo, height, text, modify);
        result.add_region(new_region);
    }
    result
}



/// A set of zero or more selection regions, representing a selection state.
#[derive(Default, Debug, Clone)]
pub struct Selection {
    // An invariant: regions[i].max() <= regions[i+1].min()
    // and < if either is_caret()
    regions: Vec<SelRegion>,
}

/// Implementing the Deref trait allows callers to easily test `is_empty`, iterate
/// through all ranges, etc.
impl Deref for Selection {
    type Target = [SelRegion];

    fn deref(&self) -> &[SelRegion] {
        &self.regions
    }
}

impl Selection {
    /// Creates a new empty selection.
    pub fn new() -> Selection {
        Selection::default()
    }

    /// Add a region to the selection. This method implements merging logic.
    ///
    /// Two non-caret regions merge if their interiors intersect; merely
    /// touching at the edges does not cause a merge. A caret merges with
    /// a non-caret if it is in the interior or on either edge. Two carets
    /// merge if they are the same offset.
    ///
    /// Performance note: should be O(1) if the new region strictly comes
    /// after all the others in the selection, otherwise O(n).
    pub fn add_region(&mut self, region: SelRegion) {
//        let mut ix = self.search(region.min());
//        if ix == self.regions.len() {
//            self.regions.push(region);
//            return;
//        }
//        let mut region = region;
//        let mut end_ix = ix;
//        if self.regions[ix].min() <= region.min() {
//            if self.regions[ix].should_merge(region) {
//                region = region.merge_with(self.regions[ix]);
//            } else {
//                ix += 1;
//            }
//            end_ix += 1;
//        }
//        while end_ix < self.regions.len() && region.should_merge(self.regions[end_ix]) {
//            region = region.merge_with(self.regions[end_ix]);
//            end_ix += 1;
//        }
//        if ix == end_ix {
//            self.regions.insert(ix, region);
//        } else {
//            self.regions[ix] = region;
//            remove_n_at(&mut self.regions, ix + 1, end_ix - ix - 1);
//        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SelRegion {
    /// The inactive edge of a selection, as a byte offset. When
    /// equal to end, the selection range acts as a caret.
    pub start: usize,

    /// The active edge of a selection, as a byte offset.
    pub end: usize,

    /// A saved horizontal position (used primarily for line up/down movement).
    pub horiz: Option<HorizPos>,

//    /// The affinity of the cursor.
//    pub affinity: Affinity,
}

impl SelRegion {

    /// Returns a new region.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end, horiz: None }//, affinity: Affinity::default() }
    }

    /// Gets the earliest offset within the region, ie the minimum of both edges.
    pub fn min(self) -> usize {
        min(self.start, self.end)
    }

    /// Gets the latest offset within the region, ie the maximum of both edges.
    pub fn max(self) -> usize {
        max(self.start, self.end)
    }

    /// Determines whether the region is a caret (ie has an empty interior).
    pub fn is_caret(self) -> bool {
        self.start == self.end
    }

    /// Returns a region with the given horizontal position.
    pub fn with_horiz(self, horiz: Option<HorizPos>) -> Self {
        Self { horiz, ..self }
    }

}


/// A type representing horizontal measurements. This is currently in units
/// that are not very well defined except that ASCII characters count as
/// 1 each. It will change.
pub type HorizPos = usize;


/// The specification of a movement.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Movement {
    /// Move to the left by one grapheme cluster.
    Left,
    /// Move to the right by one grapheme cluster.
    Right,
//    /// Move to the left by one word.
//    LeftWord,
//    /// Move to the right by one word.
//    RightWord,
    /// Move to left end of visible line.
    LeftOfLine,
    /// Move to right end of visible line.
    RightOfLine,
    /// Move up one visible line.
    Up,
    /// Move down one visible line.
    Down,
    /// Move up one viewport height.
    UpPage,
    /// Move down one viewport height.
    DownPage,
    /// Move up to the next line that can preserve the cursor position.
    UpExactPosition,
    /// Move down to the next line that can preserve the cursor position.
    DownExactPosition,
    /// Move to the start of the text line.
    StartOfParagraph,
    /// Move to the end of the text line.
    EndOfParagraph,
    /// Move to the end of the text line, or next line if already at end.
    EndOfParagraphKill,
    /// Move to the start of the document.
    StartOfDocument,
    /// Move to the end of the document
    EndOfDocument,
}


// ==================
// === LineOffset ===
// ==================

/// A trait from which lines and columns in a document can be calculated
/// into offsets inside a rope an vice versa.
pub trait LineOffset {
    // use own breaks if present, or text if not (no line wrapping)

    /// Returns the byte offset corresponding to the given line.
    fn offset_of_line(&self, text: &Rope, line: usize) -> usize {
        text.offset_of_line(line)
    }

    /// Returns the visible line number containing the given offset.
    fn line_of_offset(&self, text: &Rope, offset: usize) -> usize {
        text.line_of_offset(offset)
    }

    // How should we count "column"? Valid choices include:
    // * Unicode codepoints
    // * grapheme clusters
    // * Unicode width (so CJK counts as 2)
    // * Actual measurement in text layout
    // * Code units in some encoding
    //
    // Of course, all these are identical for ASCII. For now we use UTF-8 code units
    // for simplicity.

    fn offset_to_line_col(&self, text: &Rope, offset: usize) -> (usize, usize) {
        let line = self.line_of_offset(text, offset);
        (line, offset - self.offset_of_line(text, line))
    }

    fn line_col_to_offset(&self, text: &Rope, line: usize, col: usize) -> usize {
        let mut offset = self.offset_of_line(text, line).saturating_add(col);
        if offset >= text.len() {
            offset = text.len();
            if self.line_of_offset(text, offset) <= line {
                return offset;
            }
        } else {
            // Snap to grapheme cluster boundary
            offset = text.prev_grapheme_offset(offset + 1).unwrap();
        }

        // clamp to end of line
        let next_line_offset = self.offset_of_line(text, line + 1);
        if offset >= next_line_offset {
            if let Some(prev) = text.prev_grapheme_offset(next_line_offset) {
                offset = prev;
            }
        }
        offset
    }

//    /// Get the line range of a selected region.
//    fn get_line_range(&self, text: &Rope, region: &SelRegion) -> Range<usize> {
//        let (first_line, _) = self.offset_to_line_col(text, region.min());
//        let (mut last_line, last_col) = self.offset_to_line_col(text, region.max());
//        if last_col == 0 && last_line > first_line {
//            last_line -= 1;
//        }
//
//        first_line..(last_line + 1)
//    }
}








use crate::prelude::*;
use ensogl::data::color;
use crate::display::shape::text::glyph::font;
use crate::display::shape::text::glyph::pen::PenIterator;
use glyph::Glyph;


// ============
// === Line ===
// ============

/// A structure keeping line of glyphs with proper alignment.
///
/// Not all the glyphs in `glyphs` vector may be actually in use. This structure is meant to keep
/// changing text, and for best performance it re-uses the created Glyphs (what means the specific
/// buffer space). Therefore you can set a cap for line length by using the `set_fixed_capacity`
/// method.
#[derive(Clone,CloneRef,Debug)]
pub struct Line {
    display_object : display::object::Instance,
    glyph_system   : glyph::System,
    content        : Rc<RefCell<String>>,
    glyphs         : Rc<RefCell<Vec<Glyph>>>,
    font_color     : Rc<Cell<color::Rgba>>,
    font_size      : Rc<Cell<f32>>,
    fixed_capacity : Rc<Cell<Option<usize>>>,
}

impl Line {
    /// Constructor.
    pub fn new(logger:impl AnyLogger, glyph_system:&glyph::System) -> Self {
        let logger         = Logger::sub(logger,"line");
        let display_object = display::object::Instance::new(logger);
        let glyph_system   = glyph_system.clone_ref();
        let font_size      = Rc::new(Cell::new(11.0));
        let font_color     = Rc::new(Cell::new(color::Rgba::new(0.0,0.0,0.0,1.0)));
        let content        = default();
        let glyphs         = default();
        let fixed_capacity = default();
        Line {display_object,glyph_system,glyphs,font_size,font_color,content,fixed_capacity}
    }

    /// Replace currently visible text.
    pub fn set_text<S:Into<String>>(&self, content:S) {
        *self.content.borrow_mut() = content.into();
        self.redraw();
    }
}


// === Setters ===

#[allow(missing_docs)]
impl Line {
    pub fn set_font_color<C:Into<color::Rgba>>(&self, color:C) {
        let color = color.into();
        self.font_color.set(color);
        for glyph in &*self.glyphs.borrow() {
            glyph.set_color(color);
        }
    }

    pub fn set_font_size(&self, size:f32) {
        self.font_size.set(size);
        self.redraw();
    }

    pub fn change_fixed_capacity(&self, count:Option<usize>) {
        self.fixed_capacity.set(count);
        self.resize();
    }

    pub fn set_fixed_capacity(&self, count:usize) {
        self.change_fixed_capacity(Some(count));
    }

    pub fn unset_fixed_capacity(&self) {
        self.change_fixed_capacity(None);
    }
}


// === Getters ===

#[allow(missing_docs)]
impl Line {
    pub fn font_size(&self) -> f32 {
        self.font_size.get()
    }

    pub fn length(&self) -> usize {
        self.content.borrow().len()
    }

//    pub fn font(&self) -> font::Handle {
//        self.glyph_system.font.clone_ref()
//    }
}


// === Internal API ===

impl Line {
    /// Resizes the line to contain enough glyphs to display the full `content`. In case the
    /// `fixed_capacity` was set, it will add or remove the glyphs to match it.
    fn resize(&self) {
        let content_len        = self.content.borrow().len();
        let target_glyph_count = self.fixed_capacity.get().unwrap_or(content_len);
        let glyph_count        = self.glyphs.borrow().len();
        if target_glyph_count > glyph_count {
            let new_count  = target_glyph_count - glyph_count;
            let new_glyphs = (0..new_count).map(|_| {
                let glyph = self.glyph_system.new_glyph();
                self.add_child(&glyph);
                glyph
            });
            self.glyphs.borrow_mut().extend(new_glyphs)
        }
        if glyph_count > target_glyph_count {
            self.glyphs.borrow_mut().truncate(target_glyph_count)
        }
    }

    /// Updates properties of all glyphs, including characters they display, size, and colors.
    fn redraw(&self) {
        self.resize();

        let content     = self.content.borrow();
        let font        = self.glyph_system.font.clone_ref();
        let font_size   = self.font_size.get();
        let chars       = content.chars();
        let pen         = PenIterator::new(font_size,chars,font);
        let content_len = content.len();
        let color       = self.font_color.get();

        for (glyph,(chr,x_offset)) in self.glyphs.borrow().iter().zip(pen) {
            let glyph_info   = self.glyph_system.font.get_glyph_info(chr);
            let size         = glyph_info.scale.scale(font_size);
            let glyph_offset = glyph_info.offset.scale(font_size);
            let glyph_x      = x_offset + glyph_offset.x;
            let glyph_y      = glyph_offset.y;
            glyph.set_position(Vector3::new(glyph_x,glyph_y,0.0));
            glyph.set_glyph(chr);
            glyph.set_color(color);
            glyph.size.set(size);
        }

        for glyph in self.glyphs.borrow().iter().skip(content_len) {
            glyph.size.set(Vector2::new(0.0,0.0));
        }
    }
}


// === Display Object ===

impl display::Object for Line {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}




///// Test.
//pub fn main() {
////    let mut text = Rope::from("hello\nworld\n!!!\nyo");
////    let mut cursor = Cursor::new(&text, 0);
////
////    while cursor.pos() < text.len() - 2 {
////        cursor.next::<BaseMetric>();
////
////        println!("{:?}",cursor.pos());
////    }
////    a.edit(5..6, "!");
////    for i in 0..1000000 {
////        let l = a.len();
////        a.edit(l..l, &(i.to_string() + "\n"));
////    }
////    let l = a.len();
////    for s in a.clone().iter_chunks(1000..3000) {
////        println!("chunk {:?}", s);
////    }
////    a.edit(1000..l, "");
////    //a = a.subrange(0, 1000);
////    println!("{:?}", String::from(a));
//}