
use crate::prelude::*;
use crate::buffer::*;
use crate::buffer::location::*;



// =================
// === Constants ===
// =================

/// When paging through a file, the number of lines from the previous page that will also be visible
/// in the next.
const SCROLL_OVERLAP : isize = 2;

/// Default visible line count in a new buffer view.
const DEFAULT_LINE_COUNT : usize = 10;



// ============
// === View ===
// ============

/// View for a region of a buffer. There are several cases where multiple views share the same
/// buffer, including displaying the buffer in separate tabs or displaying multiple users in the
/// same file (keeping a view per user and merging them visually).
#[allow(missing_docs)]
pub struct View {
    buffer            : Buffer,
    first_line_number : Rc<Cell<usize>>,
    line_count        : Rc<Cell<usize>>,
    selections        : Rc<RefCell<selection::Group>>,
}







impl View {
    /// Constructor.
    pub fn new(buffer:impl Into<Buffer>) -> Self {
        let buffer = buffer.into();
        let first_line_number = default();
        let line_count = Rc::new(Cell::new(DEFAULT_LINE_COUNT));
        let mut selections = default();
//        selections.add_region(Selection::new(0,0)); // fixme: remove
        Self {buffer,first_line_number,line_count,selections}
    }

    /// Add a new selection to the current view.
    pub fn add_selection(&self, selection:impl Into<Selection>) {
        self.selections.borrow_mut().add_region(selection.into())
    }

    pub fn selections(&self) -> selection::Group {
        self.selections.borrow().clone()
    }

    /// If `modify` is `true`, the selections are modified, otherwise the results
    /// of individual region movements become carets.
    pub fn navigate(&mut self, navigation:Navigation, modify: bool) {
        self.set_selection(self.moved_selection(navigation,modify));
    }

    /// Computes the actual desired amount of scrolling (generally slightly
    /// less than the height of the viewport, to allow overlap).
    fn page_scroll_height(&self) -> isize {
        std::cmp::max(self.line_count.get() as isize - SCROLL_OVERLAP, 1)
    }

//    /// Returns the regions of the current selection.
//    pub fn sel_regions(&self) -> &[Selection] {
//        &self.selections
//    }

    /// Set the selection to a new value.
    pub fn set_selection(&mut self, selection:impl Into<selection::Group>) {
        //self.invalidate_selection();
        *self.selections.borrow_mut() = selection.into();
        //self.invalidate_selection();
//        self.scroll_to_cursor(text);
    }

    /// Sets the selection to a new value, invalidating the line cache as needed.
    /// This function does not perform any scrolling.
    fn set_selection_raw(&mut self, sel: selection::Group) {

    }

//    fn scroll_to_cursor(&mut self, text: &Rope) {
//        let end = self.sel_regions().last().unwrap().end;
//        let line = self.line_of_offset(text, end);
//        if line < self.first_line_number {
//            self.first_line_number = line;
//        } else if self.first_line_number + self.height <= line {
//            self.first_line_number = line - (self.height - 1);
//        }
//        // We somewhat arbitrarily choose the last region for setting the old-style
//        // selection state, and for scrolling it into view if needed. This choice can
//        // likely be improved.
//        self.scroll_to = Some(end);
//    }

    /// Invalidate the current selection. Note that we could be even more
    /// fine-grained in the case of multiple cursors, but we also want this
    /// method to be fast even when the selection is large.
    fn invalidate_selection(&mut self, text: &Rope) {
//        // TODO: refine for upstream (caret appears on prev line)
//        let first_line_number = self.line_of_offset(text, self.selection.first().unwrap().min());
//        let last_line = self.line_of_offset(text, self.selection.last().unwrap().max()) + 1;
//        let all_caret = self.selection.iter().all(|region| region.is_caret());
//        let invalid = if all_caret {
//            line_cache_shadow::CURSOR_VALID
//        } else {
//            line_cache_shadow::CURSOR_VALID | line_cache_shadow::STYLES_VALID
//        };
//        self.lc_shadow.partial_invalidate(first_line_number, last_line, invalid);
    }
}


impl View {
    /// Based on the current selection position this will return the cursor position, the current line, and the
/// total number of lines of the file.
    fn selection_position(&self, selection:Selection, move_up:bool, modify:bool) -> Location {
        // The active point of the selection
        let active = if modify {
            selection.end
        } else if move_up {
            selection.min()
        } else {
            selection.max()
        };
        let column = if let Some(col) = selection.column { col } else { self.offset_to_line_col(active).column };
        let line   = self.line_of_offset(active);

        Location(line,column)
    }


    /// Compute movement based on vertical motion by the given number of lines.
///
/// Note: in non-exceptional cases, this function preserves the `horiz`
/// field of the selection region.
    fn vertical_motion(&self,
                       region: Selection,
                       line_delta: isize,
                       modify: bool,
    ) -> (ByteOffset, Option<Column>) {
        let loc     = self.selection_position(region, line_delta < 0, modify);
        let n_lines = self.line_of_offset(ByteOffset(self.text().len()));

        // This code is quite careful to avoid integer overflow.
        // TODO: write tests to verify
        if line_delta < 0 && (-line_delta as usize) > loc.line.raw {
            return (ByteOffset(0), Some(loc.column));
        }
        let line = if line_delta < 0 {
            loc.line.raw - (-line_delta as usize)
        } else {
            loc.line.raw.saturating_add(line_delta as usize)
        };
        if line > n_lines.raw {
            return (ByteOffset(self.text().len()), Some(loc.column));
        }
        let line = Line(line);
        let new_offset = self.line_col_to_offset(line, loc.column);
        (new_offset, Some(loc.column))
    }

    /// Compute movement based on vertical motion by the given number of lines skipping
/// any line that is shorter than the current cursor position.
    fn vertical_motion_exact_pos(&self,
                                 region: Selection,
                                 move_up: bool,
                                 modify: bool,
    ) -> (ByteOffset,Option<Column>) {
        let loc = self.selection_position(region, move_up, modify);
        let n_lines = self.line_of_offset(ByteOffset(self.text().len()));

        let line_len = self.offset_of_line(loc.line.saturating_add(1)) - self.offset_of_line(loc.line);
        if move_up && loc.line == Line(0) {
            return (self.line_col_to_offset(loc.line, loc.column), Some(loc.column));
        }
        let mut line = if move_up { loc.line - 1 } else { loc.line.saturating_add(1) };

        // If the active columns is longer than the current line, use the current line length.
        let line_last_column = Column(line_len.raw);
        let col = if line_last_column < loc.column { line_last_column - 1 } else { loc.column };

        loop {
            let line_len = self.offset_of_line(line + 1) - self.offset_of_line(line);

            // If the line is longer than the current cursor position, break.
            // We use > instead of >= because line_len includes newline.
            if line_len.raw > col.raw {
                break;
            }

            // If you are trying to add a selection past the end of the file or before the first line, return original selection
            if line >= n_lines || (line == Line(0) && move_up) {
                line = loc.line;
                break;
            }

            line = if move_up { line - 1 } else { line.saturating_add(1) };
        }

        (self.line_col_to_offset(line, col), Some(col))
    }
}






impl View {
    /// Apply the movement to each region in the selection, and returns the union of the results.
    ///
    /// If `modify` is `true`, the selections are modified, otherwise the results of individual region
    /// movements become carets. Modify is often mapped to the `shift` button in text editors.
    pub fn moved_selection(&self, movement:Navigation, modify:bool) -> selection::Group {
        let mut result = selection::Group::new();
        for &selection in self.selections.borrow().iter() {
            let new_selection = self.moved_selection_region(movement,selection,modify);
            result.add_region(new_selection);
        }
        result
    }

    /// Compute the result of movement on one selection region.
    pub fn moved_selection_region
    (&self, movement:Navigation, region:Selection, modify:bool) -> Selection {
        let text        = self.text();
        let no_horiz    = |t|(t,None);
        let (end,horiz) : (ByteOffset,Option<Column>) = match movement {

            Navigation::Up                => self.vertical_motion(region, -1, modify),
            Navigation::Down              => self.vertical_motion(region,  1, modify),
            Navigation::UpExactPosition   => self.vertical_motion_exact_pos(region, true, modify),
            Navigation::DownExactPosition => self.vertical_motion_exact_pos(region, false, modify),
            Navigation::UpPage            => self.vertical_motion(region, -self.page_scroll_height(), modify),
            Navigation::DownPage          => self.vertical_motion(region,  self.page_scroll_height(), modify),
            Navigation::StartOfDocument   => no_horiz(ByteOffset(0)),
            Navigation::EndOfDocument     => no_horiz(ByteOffset(text.len())),

            Navigation::Left => {
                let def     = (ByteOffset(0),region.column);
                let do_move = region.is_caret() || modify;
                if  do_move { text.prev_grapheme_offset(region.end.raw).map(ByteOffset).map(no_horiz).unwrap_or(def) }
                else        { no_horiz(region.min()) }
            }

            Navigation::Right => {
                let def     = (region.end,region.column);
                let do_move = region.is_caret() || modify;
                if  do_move { text.next_grapheme_offset(region.end.raw).map(ByteOffset).map(no_horiz).unwrap_or(def) }
                else        { no_horiz(region.max()) }
            }

            Navigation::LeftOfLine => {
                let line   = self.line_of_offset(region.end);
                let offset = self.offset_of_line(line);
                no_horiz(offset)
            }

            Navigation::RightOfLine => {
                let line             = self.line_of_offset(region.end);
                let text_len         = ByteOffset(text.len());
                let last_line        = line == self.line_of_offset(text_len);
                let next_line_offset = self.offset_of_line(line+1);
                let offset           = if last_line { text_len } else {
                    text.prev_grapheme_offset(next_line_offset.raw).map(ByteOffset).unwrap_or(text_len)
                };
                no_horiz(offset)
            }

            Navigation::StartOfParagraph => {
                // Note: TextEdit would start at modify ? region.end : region.min()
                let mut cursor = rope::Cursor::new(&text,region.end.raw);
                let offset     = ByteOffset(cursor.prev::<rope::LinesMetric>().unwrap_or(0));
                no_horiz(offset)
            }

            Navigation::EndOfParagraph => {
                // Note: TextEdit would start at modify ? region.end : region.max()
                let mut cursor = rope::Cursor::new(&text,region.end.raw);
                let     offset = match cursor.next::<rope::LinesMetric>() {
                    None            => ByteOffset(text.len()),
                    Some(next_line) => {
                        if cursor.is_boundary::<rope::LinesMetric>() {
                            text.prev_grapheme_offset(next_line).map(ByteOffset).unwrap_or(region.end)
                        } else if cursor.pos() == text.len() {
                            ByteOffset(text.len())
                        } else {
                            region.end
                        }
                    }
                };
                no_horiz(offset)
            }

            Navigation::EndOfParagraphKill => {
                // Note: TextEdit would start at modify ? region.end : region.max()
                let mut cursor = rope::Cursor::new(&text,region.end.raw);
                let     offset = match cursor.next::<rope::LinesMetric>() {
                    None            => region.end,
                    Some(next_line) => {
                        if cursor.is_boundary::<rope::LinesMetric>() {
                            let eol = text.prev_grapheme_offset(next_line);
                            let opt = eol.and_then(|t|(t!=region.end.raw).as_some(t));
                            let off = opt.unwrap_or(next_line);
                            ByteOffset(off)
                        } else { ByteOffset(next_line) }
                    }
                };
                no_horiz(offset)
            }
        };
        let start = if modify { region.start } else { end };
        Selection::new(start,end).with_column(horiz)
    }
}


impl LineOffset for View {
    fn text(&self) -> &Rope {
        &self.buffer.rope
    }

    fn offset_of_line(&self,line:Line) -> ByteOffset {
        let line = std::cmp::min(line.raw,self.text().measure::<rope::LinesMetric>() + 1);
        ByteOffset(self.text().offset_of_line(line))
    }

    fn line_of_offset(&self,offset:ByteOffset) -> Line {
        Line(self.text().line_of_offset(offset.raw))
    }
}





// ==================
// === LineOffset ===
// ==================

/// A trait from which lines and columns in a document can be calculated
/// into offsets inside a rope an vice versa.
pub trait LineOffset {
    // use own breaks if present, or text if not (no line wrapping)

    fn text(&self) -> &Rope;

    /// Returns the byte offset corresponding to the given line.
    fn offset_of_line(&self, line:Line) -> ByteOffset {
        ByteOffset(self.text().offset_of_line(line.raw))
    }

    /// Returns the visible line number containing the given offset.
    fn line_of_offset(&self, offset:ByteOffset) -> Line {
        Line(self.text().line_of_offset(offset.raw))
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

    fn offset_to_line_col(&self, offset:ByteOffset) -> Location {
        let line = self.line_of_offset(offset);
        let col  = (offset - self.offset_of_line(line)).as_column();
        Location(line,col)
    }

    fn line_col_to_offset(&self, line:Line, col:Column) -> ByteOffset {
        let mut offset = self.offset_of_line(line).saturating_add(col.raw);
        let len = ByteOffset(self.text().len());
        if offset >= len {
            offset = len;
            if self.line_of_offset(offset) <= line {
                return offset;
            }
        } else {
            // Snap to grapheme cluster boundary
            offset = ByteOffset(self.text().prev_grapheme_offset(offset.raw + 1).unwrap());
        }

        // clamp to end of line
        let next_line_offset = self.offset_of_line(line + 1);
        if offset >= next_line_offset {
            if let Some(prev) = self.text().prev_grapheme_offset(next_line_offset.raw) {
                offset = ByteOffset(prev);
            }
        }
        offset
    }

//    /// Get the line range of a selected region.
//    fn get_line_range(&self, text: &Rope, region: &Selection) -> Range<usize> {
//        let (first_line_number, _) = self.offset_to_line_col(text, region.min());
//        let (mut last_line, last_col) = self.offset_to_line_col(text, region.max());
//        if last_col == 0 && last_line > first_line_number {
//            last_line -= 1;
//        }
//
//        first_line_number..(last_line + 1)
//    }
}
