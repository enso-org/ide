//! Text cursor transform implementation.

use super::*;
use crate::buffer::data;
use crate::buffer::data::unit::*;
use crate::buffer::view::word::WordCursor;



// =================
// === Transform ===
// =================

/// Selection transformation patterns. Used for the needs of keyboard and mouse interaction.
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Transform {
    /// Select all text.
    All,
    /// Move to the left by one grapheme cluster.
    Left,
    /// Move to the right by one grapheme cluster.
    Right,
    /// Move to the left selection border. Cursors will not be modified.
    LeftSelectionBorder,
    /// Move to the right selection border. Cursors will not be modified.
    RightSelectionBorder,
    /// Move to the left by one word.
    LeftWord,
    /// Move to the right by one word.
    RightWord,
    /// Select the word at every cursor.
    Word,
    /// Move to left end of visible line.
    LeftOfLine,
    /// Move to right end of visible line.
    RightOfLine,
    /// Move up one visible line.
    Up,
    /// Move down one visible line.
    Down,
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



// ==========================
// === Transform Handling ===
// ==========================

impl ViewBuffer {
    /// Convert selection to caret location after a vertical movement.
    fn vertical_motion_selection_to_caret
    (&self, selection:Selection, move_up:bool, modify:bool) -> Location {
        let offset = if modify { selection.end } else if move_up { selection.min() } else { selection.max() };
        self.offset_to_line_col(offset)

//        let column   = selection.column.unwrap_or(location.column);
//        Location(location.line,column)
    }

    /// Compute movement based on vertical motion by the given number of lines.
    fn vertical_motion
    (&self, selection:Selection, line_delta:Line, modify:bool) -> (Bytes,Bytes,Option<Column>) {
        let move_up    = line_delta < 0.line();
        let line_delta = line_delta.abs();
        let location   = self.vertical_motion_selection_to_caret(selection, move_up, modify);
        let n_lines    = self.line_of_offset(self.data().len());

        if move_up && line_delta > location.line {
            return (selection.start,Bytes(0),None); // FIXME None -> Some(location.offset)
        }

        let line = if move_up { location.line - line_delta } else { location.line.saturating_add(line_delta) };
        let column = self.column_of_location(location.line, location.offset);
        let tgt_offset = self.line_offset_of_location_X(line,column);

        if line > n_lines {
            return (selection.start,self.data().len(),None) // FIXME None -> Some(location.offset)
        }

        let new_offset = self.line_col_to_offset(line, tgt_offset);
        (selection.start,new_offset,None) // FIXME None -> Some(location.offset)
    }

    fn column_of_location(&self, line:Line, line_offset:Bytes) -> Column {
        let mut offset = self.offset_of_line(line);
        let tgt_offset = offset + line_offset;
        let mut column = 0.column();
        while offset < tgt_offset {
            match self.next_grapheme_offset(offset) {
                None => break,
                Some(off) => {
                    column += 1.column();
                    offset = off;
                }
            }
        }
        column
    }

    fn line_offset_of_location_X(&self, line:Line, line_column:Column) -> Bytes {
        let start_offset = self.offset_of_line(line);
        let mut offset = start_offset;
        let mut column = 0.column();
        while column < line_column {
            match self.next_grapheme_offset(offset) {
                None => break,
                Some(off) => {
                    column += 1.column();
                    offset = off;
                }
            }
        }
        offset - start_offset
    }

    /// Compute movement based on vertical motion by the given number of lines skipping
    /// any line that is shorter than the current cursor position.
    fn vertical_motion_exact_pos
    (&self, region: Selection, move_up: bool, modify: bool) -> (Bytes,Bytes,Option<Column>) {
        let location = self.vertical_motion_selection_to_caret(region, move_up, modify);
        let lines_count = self.line_of_offset(self.data().len());
        let line_offset = self.offset_of_line(location.line);
        let next_line_offset = self.offset_of_line(location.line.saturating_add(1.line()));
        let line_len = next_line_offset - line_offset;
        if move_up && location.line == Line(0) {
            return (region.start,self.line_col_to_offset(location.line, location.offset),None) // FIXME None -> Some(location.offset)
        }
        let mut line = if move_up { location.line - 1.line() } else { location.line.saturating_add(1.line()) };

        // If the active columns is longer than the current line, use the current line length.
        let line_last_column = line_len;
        let col = if line_last_column < location.offset { line_last_column - 1.bytes() } else { location.offset };

        loop {
            let line_len = self.offset_of_line(line + 1.line()) - self.offset_of_line(line);

            // If the line is longer than the current cursor position, break.
            // We use > instead of >= because line_len includes newline.
            if line_len > col {
                break;
            }

            // If you are trying to add a selection past the end of the file or before the first line, return original selection
            if line >= lines_count || (line == Line(0) && move_up) {
                line = location.line;
                break;
            }

            line = if move_up { line - 1.line() } else { line.saturating_add(1.line()) };
        }

        (region.start,self.line_col_to_offset(line, col),None) // FIXME None -> Some(col)
    }

    /// Apply the movement to each region in the selection, and returns the union of the results.
    ///
    /// If `modify` is `true`, the selections are modified, otherwise the results of individual region
    /// movements become carets. Modify is often mapped to the `shift` button in text editors.
    pub fn moved_selection(&self, movement: Transform, modify: bool) -> selection::Group {
        let mut result = selection::Group::new();
        for &selection in self.selection.borrow().iter() {
            let new_selection = self.moved_selection_region(movement, selection, modify);
            result.add(new_selection);
        }
        result
    }

    pub fn selection_after_insert(&self, bytes: Bytes) -> selection::Group {
        let mut result = selection::Group::new();
        let mut offset = bytes;
        for &selection in self.selection.borrow().iter() {
            let new_selection = selection.map(|t| t + offset);
            offset += bytes;
            result.add(new_selection);
        }
        result
    }

    /// Compute the result of movement on one selection region.
    pub fn moved_selection_region
    (&self, movement:Transform, region:Selection, modify:bool) -> Selection {
        let text        = &self.data();
        let no_horiz    = |s,t|(s,t,None);
        let (start,end,horiz) : (Bytes,Bytes,Option<Column>) = match movement {
            Transform::All               => no_horiz(0.bytes(),text.len()),
            Transform::Up                => self.vertical_motion(region, -1.line(), modify),
            Transform::Down              => self.vertical_motion(region,  1.line(), modify),
            Transform::UpExactPosition   => self.vertical_motion_exact_pos(region, true, modify),
            Transform::DownExactPosition => self.vertical_motion_exact_pos(region, false, modify),
            Transform::StartOfDocument   => no_horiz(region.start,Bytes(0)),
            Transform::EndOfDocument     => no_horiz(region.start,text.len()),

            Transform::Left => {
                let def     = (region.start,Bytes(0),region.column);
                let do_move = region.is_caret() || modify;
                if  do_move { text.prev_grapheme_offset(region.end).map(|t|no_horiz(region.start,t)).unwrap_or(def) }
                else        { no_horiz(region.start,region.min()) }
            }

            Transform::Right => {
                let def     = (region.start,region.end,region.column);
                let do_move = region.is_caret() || modify;
                if  do_move { text.next_grapheme_offset(region.end).map(|t|no_horiz(region.start,t)).unwrap_or(def) }
                else        { no_horiz(region.start,region.max()) }
            }

            Transform::LeftSelectionBorder => {
                no_horiz(region.start,region.min())
            }

            Transform::RightSelectionBorder => {
                no_horiz(region.start,region.max())
            }

            Transform::LeftOfLine => {
                let line   = self.line_of_offset(region.end);
                let offset = self.offset_of_line(line);
                no_horiz(region.start,offset)
            }

            Transform::RightOfLine => {
                let line             = self.line_of_offset(region.end);
                let text_len         = text.len();
                let last_line        = line == self.line_of_offset(text_len);
                let next_line_offset = self.offset_of_line(line+1.line());
                let offset           = if last_line { text_len } else {
                    text.prev_grapheme_offset(next_line_offset).unwrap_or(text_len)
                };
                no_horiz(region.start,offset)
            }

            Transform::StartOfParagraph => {
                // Note: TextEdit would start at modify ? region.end : region.min()
                let mut cursor = data::Cursor::new(&text, region.end.value as usize);
                let offset     = cursor.prev::<data::metric::Lines>().unwrap_or(0).into();
                no_horiz(region.start,offset)
            }

            Transform::EndOfParagraph => {
                // Note: TextEdit would start at modify ? region.end : region.max()
                let mut cursor = data::Cursor::new(&text, region.end.value as usize);
                let     offset = match cursor.next::<data::metric::Lines>() {
                    None            => text.len(),
                    Some(next_line_offset) => {
                        let next_line_offset   = next_line_offset.into();
                        let cursor_pos : Bytes = cursor.pos().into();
                        if cursor.is_boundary::<data::metric::Lines>() {
                            text.prev_grapheme_offset(next_line_offset).unwrap_or(region.end)
                        } else if cursor_pos == text.len() {
                            text.len()
                        } else {
                            region.end
                        }
                    }
                };
                no_horiz(region.start,offset)
            }

            Transform::EndOfParagraphKill => {
                // Note: TextEdit would start at modify ? region.end : region.max()
                let mut cursor = data::Cursor::new(&text, region.end.value as usize);
                let     offset = match cursor.next::<data::metric::Lines>() {
                    None            => region.end,
                    Some(next_line_offset) => {
                        let next_line_offset : Bytes = next_line_offset.into();
                        if cursor.is_boundary::<data::metric::Lines>() {
                            let eol = text.prev_grapheme_offset(next_line_offset);
                            let opt = eol.and_then(|t|(t!=region.end).as_some(t));
                            opt.unwrap_or(next_line_offset)
                        } else { next_line_offset }
                    }
                };
                no_horiz(region.start,offset)
            }

            Transform::LeftWord => {
                let mut word_cursor = WordCursor::new(text,region.end);
                let offset = word_cursor.prev_boundary().unwrap_or(0.bytes());
                (region.start,offset, None)
            }

            Transform::RightWord => {
                let mut word_cursor = WordCursor::new(text,region.end);
                let offset = word_cursor.next_boundary().unwrap_or_else(|| text.len());
                (region.start,offset, None)
            }

            Transform::Word => {
                let mut word_cursor = WordCursor::new(text,region.end);
                let (start,end) = word_cursor.select_word();
                (start,end,None)
            }
        };
        let start = if modify { start } else { end };
        Selection::new(start,end,region.id).with_column(None) // FIXME None -> horiz
    }
}
