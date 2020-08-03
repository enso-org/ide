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
        let end = selection.end;
        if modify {end} else if move_up {selection.min()} else {selection.max()}
    }

    /// Compute movement based on vertical motion by the given number of lines.
    fn vertical_motion
    (&self, selection:Selection, line_delta:Line, modify:bool) -> (Location,Location) {
        let move_up       = line_delta < 0.line();
        let location      = self.vertical_motion_selection_to_caret(selection,move_up,modify);
        let line          = location.line + line_delta;
        if line < 0.line() {
            (selection.start,default()) // FIXME None -> Some(location.offset)
        } else if line > self.last_line() {
            let max_location = self.offset_to_location(self.data().len());
            (selection.start,max_location) // FIXME None -> Some(location.offset)
        } else {
            let tgt_location = location.with_line(line);
            // let new_offset = self.line_offset_of_location_X2(tgt_location);
            (selection.start, tgt_location) // FIXME None -> Some(location.offset)
        }
    }

    fn last_line(&self) -> Line {
        self.line_of_offset(self.data().len())
    }

    fn last_offset(&self) -> Bytes {
        self.data().len()
    }

    pub fn column_of_location_X(&self, line:Line, line_offset:Bytes) -> Column {
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

    pub fn line_offset_of_location_X(&self, location:Location) -> Bytes {
        let start_offset = self.offset_of_line(location.line);
        let mut offset = start_offset;
        let mut column = 0.column();
        while column < location.column {
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

    pub fn line_offset_of_location_X2(&self, location:Location) -> Option<Bytes> {
        let line_offset      = self.offset_of_line2(location.line)?;
        let next_line_offset = self.offset_of_line2(location.line + 1.line());
        println!("next_line_offset {:?}",next_line_offset);
        let max_offset       = next_line_offset.and_then(|t|self.prev_grapheme_offset(t)).unwrap_or_else(||self.last_offset());
        println!("max_offset {:?}",max_offset);
        let mut offset = line_offset;
        let mut column = 0.column();
        while column < location.column {
            match self.next_grapheme_offset(offset) {
                None => break,
                Some(off) => {
                    column += 1.column();
                    offset = off;
                }
            }
        }
        Some(offset.min(max_offset))
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

//    pub fn selection_after_insert(&self, bytes: Bytes) -> selection::Group {
//        let mut result = selection::Group::new();
//        let mut offset = bytes;
//        for &selection in self.selection.borrow().iter() {
//            let new_selection = selection.map(|t| t + offset);
//            offset += bytes;
//            result.add(new_selection);
//        }
//        result
//    }

    pub fn prev_grapheme_location(&self, location:Location) -> Option<Location> {
        let offset      = self.line_col_to_offset(location)?;
        let prev_offset = self.prev_grapheme_offset(offset);
        let out = prev_offset.map(|off| self.offset_to_location(off));
        println!("!!> {:?} {:?}", offset, prev_offset);
        println!("!!! {:?} {:?}", location, out);
        out
    }

    pub fn next_grapheme_location(&self, location:Location) -> Option<Location> {
        let offset      = self.line_col_to_offset(location)?;
        let next_offset = self.next_grapheme_offset(offset);
        next_offset.map(|off| self.offset_to_location(off))
    }

    /// Compute the result of movement on one selection region.
    pub fn moved_selection_region
    (&self, movement:Transform, region:Selection, modify:bool) -> Selection {
        let text        = &self.data();
        let (start,end) : (Location,Location) = match movement {
            Transform::All               => (default(),self.offset_to_location(text.len())),
            Transform::Up                => self.vertical_motion(region, -1.line(), modify),
            Transform::Down              => self.vertical_motion(region,  1.line(), modify),
//            Transform::UpExactPosition   => self.vertical_motion_exact_pos(region, true, modify),
//            Transform::DownExactPosition => self.vertical_motion_exact_pos(region, false, modify),
            Transform::StartOfDocument   => (region.start,default()),
            Transform::EndOfDocument     => (region.start,self.offset_to_location(text.len())),

            Transform::Left => {
                let def     = (region.start,default());
                let do_move = region.is_caret() || modify;
                if  do_move { self.prev_grapheme_location(region.end).map(|t|(region.start,t)).unwrap_or(def) }
                else        { (region.start,region.min()) }
            }

            Transform::Right => {
                let def     = (region.start,region.end);
                let do_move = region.is_caret() || modify;
                if  do_move { self.next_grapheme_location(region.end).map(|t|(region.start,t)).unwrap_or(def) }
                else        { (region.start,region.max()) }
            }

            Transform::LeftSelectionBorder => {
                (region.start,region.min())
            }

            Transform::RightSelectionBorder => {
                (region.start,region.max())
            }

            Transform::LeftOfLine => {
                let end = Location(region.end.line,0.column());
                (region.start,end)
            }

            Transform::RightOfLine => {
                let line             = region.end.line;
                let text_len         = text.len();
                let is_last_line     = line == self.last_line();
                let next_line_offset = self.offset_of_line(line+1.line());
                let offset           = if is_last_line { text_len } else {
                    text.prev_grapheme_offset(next_line_offset).unwrap_or(text_len)
                };
                let end = self.offset_to_location(offset);
                (region.start,end)
            }

            Transform::LeftWord => {
                let end_offset      = self.line_col_to_offset(region.end).unwrap_or_default();
                let mut word_cursor = WordCursor::new(text,end_offset);
                let offset          = word_cursor.prev_boundary().unwrap_or(0.bytes());
                let end             = self.offset_to_location(offset);
                (region.start,end)
            }

            Transform::RightWord => {
                let end_offset      = self.line_col_to_offset(region.end).unwrap_or_default();
                let mut word_cursor = WordCursor::new(text,end_offset);
                let offset          = word_cursor.next_boundary().unwrap_or_else(|| text.len());
                let end             = self.offset_to_location(offset);
                (region.start,end)
            }

            Transform::Word => {
                let end_offset      = self.line_col_to_offset(region.end).unwrap_or_default();
                let mut word_cursor = WordCursor::new(text,end_offset);
                let offsets         = word_cursor.select_word();
                let start           = self.offset_to_location(offsets.0);
                let end             = self.offset_to_location(offsets.1);
                (start,end)
            }
        };
        let start = if modify { start } else { end };
        Selection::new(start,end,region.id) // FIXME None -> horiz
    }
}
