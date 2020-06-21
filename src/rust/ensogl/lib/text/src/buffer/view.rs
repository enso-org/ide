
use crate::prelude::*;
use crate::buffer;
use crate::buffer::*;



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
#[derive(Debug,Clone,CloneRef)]
pub struct View {
    pub buffer        : Buffer,
    first_line_number : Rc<Cell<Line>>,
    line_count        : Rc<Cell<usize>>,
    selections        : Rc<RefCell<selection::Group>>,
}

impl Deref for View {
    type Target = Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl View {
    /// Constructor.
    pub fn new(buffer:impl Into<Buffer>) -> Self {
        let buffer            = buffer.into();
        let first_line_number = default();
        let line_count        = Rc::new(Cell::new(DEFAULT_LINE_COUNT));
        let mut selections    = default();
        Self {buffer,first_line_number,line_count,selections}
    }

    /// Add a new selection to the current view.
    pub fn add_selection(&self, selection:impl Into<Selection>) {
        self.selections.borrow_mut().add_region(selection.into())
    }

    /// Set the selection to a new value.
    pub fn set_selection(&self, selection:impl Into<selection::Group>) {
        *self.selections.borrow_mut() = selection.into();
    }

    /// Return all active selections.
    pub fn selections(&self) -> selection::Group {
        self.selections.borrow().clone()
    }

    /// Move all carets by the provided movement. All selections will be converted to carets.
    pub fn move_carets(&self, movement:Movement) {
        self.move_carets_or_modify_selections(movement,false)
    }

    /// Modify the selections by the provided movement. All carets will be converted to selections.
    pub fn modify_selections(&self, movement:Movement) {
        self.move_carets_or_modify_selections(movement,true)
    }

    /// If `modify` is `true`, the selections are modified, otherwise the results of individual
    /// region movements become carets.
    fn move_carets_or_modify_selections(&self, movement:Movement, modify: bool) {
        self.set_selection(self.moved_selection(movement,modify));
    }

    /// Computes the actual desired amount of scrolling (generally slightly less than the height of
    /// the viewport, to allow overlap).
    fn page_scroll_height(&self) -> isize {
        std::cmp::max(self.line_count.get() as isize - SCROLL_OVERLAP, 1)
    }

    pub fn first_line_number(&self) -> Line {
        self.first_line_number.get()
    }

    pub fn last_line_number(&self) -> Line {
        self.first_line_number() + self.line_count()
    }

    pub fn line_count(&self) -> usize {
        self.line_count.get()
    }

    pub fn line_range(&self) -> Range<Line> {
        self.first_line_number() .. self.last_line_number()
    }

    pub fn first_line_offset(&self) -> Bytes {
        self.offset_of_line(self.first_line_number())
    }

    pub fn last_line_offset(&self) -> Bytes {
        self.offset_of_line(self.last_line_number())
    }

    pub fn line_offset_range(&self) -> Range<Bytes> {
        self.first_line_offset() .. self.last_line_offset()
    }

    pub fn offset_of_view_line(&self, view_line:Line) -> Bytes {
        let line = self.first_line_number() + view_line;
        self.offset_of_line(line)
    }

    // FIXME: this sohuld not include line break.
    pub fn range_of_view_line_raw(&self, view_line:Line) -> Range<Bytes> {
        let start = self.offset_of_view_line(view_line);
        let end   = self.offset_of_view_line(view_line + 1);
        start .. end
    }

//    pub fn get(&self, line:Line) -> String {
//        let last_line_number = self.line_of_offset(self.data().len());
//        let start   = self.offset_of_line(line);
//        let end     = self.offset_of_line(line+1);
//        let end     = self.buffer.text.prev_grapheme_offset(end).unwrap_or(end);
//        let content = self.buffer.text.rope.subseq(start.raw .. end.raw);
//        println!("buffer line count: {}", last_line_number.raw);
//        content.into()
//    }

    pub fn lines(&self) -> buffer::Lines {
        let range = self.line_offset_range();
        self.buffer.data.rope.lines(range.start.raw .. range.end.raw)
    }

//    fn scroll_to_cursor(&mut self, text: &Text) {
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

}


impl View {
    /// Convert selection to caret location after a vertical movement.
    fn vertical_motion_selection_to_caret
    (&self, selection:Selection, move_up:bool, modify:bool) -> Location {
        let offset =
            if      modify  { selection.end }
            else if move_up { selection.min() }
            else            { selection.max() };
        let location = self.offset_to_line_col(offset);
        let column   = selection.column.unwrap_or(location.column);
        Location(location.line,column)
    }


    /// Compute movement based on vertical motion by the given number of lines.
    ///
    /// Note: in non-exceptional cases, this function preserves the `column` field of the selection
    /// region.
    ///
    /// Note: This code is quite careful to avoid integer overflow.
    // TODO: Write tests to verify that it's safe regarding integer ovewrflow.
    fn vertical_motion
    (&self, region:Selection, line_delta:isize, modify:bool) -> (Bytes,Option<Column>) {
        let move_up    = line_delta < 0;
        let line_delta = line_delta.saturating_abs() as usize;
        let location   = self.vertical_motion_selection_to_caret(region,move_up,modify);
        let n_lines    = self.line_of_offset(self.data().len());

        if move_up && line_delta > location.line.raw {
            return (Bytes(0), Some(location.column));
        }

        let line = if move_up { location.line.raw - line_delta }
                   else       { location.line.raw.saturating_add(line_delta) };

        if line > n_lines.raw {
            return (self.data().len(),Some(location.column));
        }

        let line = Line(line);
        let new_offset = self.line_col_to_offset(line,location.column);
        (new_offset,Some(location.column))
    }

    /// Compute movement based on vertical motion by the given number of lines skipping
    /// any line that is shorter than the current cursor position.
    fn vertical_motion_exact_pos
    (&self, region:Selection, move_up:bool, modify:bool) -> (Bytes,Option<Column>) {
        let location    = self.vertical_motion_selection_to_caret(region, move_up, modify);
        let lines_count = self.line_of_offset(self.data().len());

        let line_len = self.offset_of_line(location.line.saturating_add(1)) - self.offset_of_line(location.line);
        if move_up && location.line == Line(0) {
            return (self.line_col_to_offset(location.line, location.column), Some(location.column));
        }
        let mut line = if move_up { location.line - 1 } else { location.line.saturating_add(1) };

        // If the active columns is longer than the current line, use the current line length.
        let line_last_column = Column(line_len.raw);
        let col = if line_last_column < location.column { line_last_column - 1 } else { location.column };

        loop {
            let line_len = self.offset_of_line(line + 1) - self.offset_of_line(line);

            // If the line is longer than the current cursor position, break.
            // We use > instead of >= because line_len includes newline.
            if line_len.raw > col.raw {
                break;
            }

            // If you are trying to add a selection past the end of the file or before the first line, return original selection
            if line >= lines_count || (line == Line(0) && move_up) {
                line = location.line;
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
    pub fn moved_selection(&self, movement:Movement, modify:bool) -> selection::Group {
        let mut result = selection::Group::new();
        for &selection in self.selections.borrow().iter() {
            let new_selection = self.moved_selection_region(movement,selection,modify);
            result.add_region(new_selection);
        }
        result
    }

    /// Compute the result of movement on one selection region.
    pub fn moved_selection_region
    (&self, movement:Movement, region:Selection, modify:bool) -> Selection {
        let text        = self.data();
        let no_horiz    = |t|(t,None);
        let (end,horiz) : (Bytes,Option<Column>) = match movement {

            Movement::Up                => self.vertical_motion(region, -1, modify),
            Movement::Down              => self.vertical_motion(region,  1, modify),
            Movement::UpExactPosition   => self.vertical_motion_exact_pos(region, true, modify),
            Movement::DownExactPosition => self.vertical_motion_exact_pos(region, false, modify),
            Movement::UpPage            => self.vertical_motion(region, -self.page_scroll_height(), modify),
            Movement::DownPage          => self.vertical_motion(region,  self.page_scroll_height(), modify),
            Movement::StartOfDocument   => no_horiz(Bytes(0)),
            Movement::EndOfDocument     => no_horiz(text.len()),

            Movement::Left => {
                let def     = (Bytes(0),region.column);
                let do_move = region.is_caret() || modify;
                if  do_move { text.prev_grapheme_offset(region.end).map(no_horiz).unwrap_or(def) }
                else        { no_horiz(region.min()) }
            }

            Movement::Right => {
                let def     = (region.end,region.column);
                let do_move = region.is_caret() || modify;
                if  do_move { text.next_grapheme_offset(region.end).map(no_horiz).unwrap_or(def) }
                else        { no_horiz(region.max()) }
            }

            Movement::LeftOfLine => {
                let line   = self.line_of_offset(region.end);
                let offset = self.offset_of_line(line);
                no_horiz(offset)
            }

            Movement::RightOfLine => {
                let line             = self.line_of_offset(region.end);
                let text_len         = text.len();
                let last_line        = line == self.line_of_offset(text_len);
                let next_line_offset = self.offset_of_line(line+1);
                let offset           = if last_line { text_len } else {
                    text.prev_grapheme_offset(next_line_offset).unwrap_or(text_len)
                };
                no_horiz(offset)
            }

            Movement::StartOfParagraph => {
                // Note: TextEdit would start at modify ? region.end : region.min()
                let mut cursor = data::Cursor::new(&text, region.end.raw);
                let offset     = Bytes(cursor.prev::<data::LinesMetric>().unwrap_or(0));
                no_horiz(offset)
            }

            Movement::EndOfParagraph => {
                // Note: TextEdit would start at modify ? region.end : region.max()
                let mut cursor = data::Cursor::new(&text, region.end.raw);
                let     offset = match cursor.next::<data::LinesMetric>() {
                    None            => text.len(),
                    Some(next_line_offset) => {
                        let next_line_offset = Bytes(next_line_offset);
                        if cursor.is_boundary::<data::LinesMetric>() {
                            text.prev_grapheme_offset(next_line_offset).unwrap_or(region.end)
                        } else if Bytes(cursor.pos()) == text.len() {
                            text.len()
                        } else {
                            region.end
                        }
                    }
                };
                no_horiz(offset)
            }

            Movement::EndOfParagraphKill => {
                // Note: TextEdit would start at modify ? region.end : region.max()
                let mut cursor = data::Cursor::new(&text, region.end.raw);
                let     offset = match cursor.next::<data::LinesMetric>() {
                    None            => region.end,
                    Some(next_line_offset) => {
                        let next_line_offset = Bytes(next_line_offset);
                        if cursor.is_boundary::<data::LinesMetric>() {
                            let eol = text.prev_grapheme_offset(next_line_offset);
                            let opt = eol.and_then(|t|(t!=region.end).as_some(t));
                            opt.unwrap_or(next_line_offset)
                        } else { next_line_offset }
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
    fn data(&self) -> &Data {
        &self.buffer.data
    }

    fn offset_of_line(&self,line:Line) -> Bytes {
        let line = std::cmp::min(line.raw,self.data().measure::<data::LinesMetric>() + 1);
        Bytes(self.data().offset_of_line(line))
    }

    fn line_of_offset(&self,offset:Bytes) -> Line {
        Line(self.data().line_of_offset(offset.raw))
    }
}





// ==================
// === LineOffset ===
// ==================

/// A trait from which lines and columns in a document can be calculated
/// into offsets inside a text an vice versa.
pub trait LineOffset {
    // use own breaks if present, or text if not (no line wrapping)

    fn data(&self) -> &Data;

    /// Returns the byte offset corresponding to the given line.
    fn offset_of_line(&self, line:Line) -> Bytes {
        Bytes(self.data().offset_of_line(line.raw))
    }

    /// Returns the visible line number containing the given offset.
    fn line_of_offset(&self, offset:Bytes) -> Line {
        Line(self.data().line_of_offset(offset.raw))
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

    fn offset_to_line_col(&self, offset:Bytes) -> Location {
        let line = self.line_of_offset(offset);
        let col  = (offset - self.offset_of_line(line)).as_column();
        Location(line,col)
    }

    fn line_col_to_offset(&self, line:Line, col:Column) -> Bytes {
        let mut offset = self.offset_of_line(line).saturating_add(col.raw);
        let len = self.data().len();
        if offset >= len {
            offset = len;
            if self.line_of_offset(offset) <= line {
                return offset;
            }
        } else {
            // Snap to grapheme cluster boundary
            offset = self.data().prev_grapheme_offset(offset + 1).unwrap_or_default();
        }

        // clamp to end of line
        let next_line_offset = self.offset_of_line(line + 1);
        if offset >= next_line_offset {
            if let Some(prev) = self.data().prev_grapheme_offset(next_line_offset) {
                offset = prev;
            }
        }
        offset
    }

//    /// Get the line range of a selected region.
//    fn get_line_range(&self, text: &Text, region: &Selection) -> Range<usize> {
//        let (first_line_number, _) = self.offset_to_line_col(text, region.min());
//        let (mut last_line, last_col) = self.offset_to_line_col(text, region.max());
//        if last_col == 0 && last_line > first_line_number {
//            last_line -= 1;
//        }
//
//        first_line_number..(last_line + 1)
//    }
}
