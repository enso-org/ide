//! The data hold by the text buffer. Under the hood it is implemented as an efficient string rope.

use crate::prelude::*;

use super::rope;
use super::rope::Rope;
use super::unit::*;
use super::range::Range;
use super::range::RangeBounds;
use super::super::view::selection::Selection; // FIXME layout



// ============
// === Text ===
// ============

/// Efficient text container used by the text buffer. Implemented as a rope under the hood.
///
/// A [rope](https://en.wikipedia.org/wiki/Rope_(data_structure)) is a data structure for strings,
/// specialized for incremental editing operations. Most operations (such as insert, delete,
/// substring) are O(log n). This module provides an immutable (also known as
/// [persistent](https://en.wikipedia.org/wiki/Persistent_data_structure)) version of Ropes, and if
/// there are many copies of similar strings, the common parts are shared.
///
/// Internally, the implementation uses thread safe reference counting. Mutations are generally
/// copy-on-write, though in-place edits are supported as an optimization when only one reference
/// exists, making the implementation as efficient as a mutable version.
///
/// This type provides multiple `From` implementations for easy conversions from string-like types,
/// and vice-versa.
///
/// Please note that the underlying rope implementation comes from `xi-rope` crate which does not
/// use strong types for all units (like line number, column number, byte offset), so part of
/// responsibility of this struct is to wrap the underlying API with strong types introduced in this
/// library.
#[derive(Debug,Clone,Default,Deref)]
#[allow(missing_docs)]
pub struct Text {
    pub rope : Rope,
}
impl_clone_ref_as_clone!(Text);


impl Text {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Check whether the text is empty.
    pub fn is_empty(&self) -> bool {
        self.rope.is_empty()
    }

    /// The number of grapheme clusters in this text.
    pub fn grapheme_count(&self) -> usize {
        let mut offset = 0;
        let mut count  = 0;
        loop {
            match self.rope.next_grapheme_offset(offset) {
                None      => break,
                Some(off) => {
                    offset = off;
                    count += 1;
                }
            }
        }
        return count
    }

    /// The first valid line index in this text.
    pub fn first_line(&self) -> Line {
        0.line()
    }

    /// The last valid line index in this text.
    pub fn last_line(&self) -> Line {
        (self.rope.measure::<rope::metric::Lines>()).into()
    }

    /// Return the len of the text in bytes.
    pub fn byte_size(&self) -> Bytes {
        Bytes(self.rope.len() as i32)
    }

    /// Range of the text in bytes.
    pub fn byte_range(&self) -> Range<Bytes> {
        (..self.byte_size()).into()
    }

    /// Constraint the provided range so it will be contained of the range of this data. This ensures that
    /// the provided range will be valid for operations on this data.
    pub fn clamp_range(&self, range:impl RangeBounds) -> Range<Bytes> {
        range.with_upper_bound(self.byte_size())
    }

//    pub fn clamp_selection(&self, selection:Selection) -> Selection {
//        let min_line = 0.line();
//        let max_line = self.last_line();
//        let max_loc  = self.end_location();
//        let start    = selection.start;
//        let start    = if selection.start.line < min_line { default() } else { start };
//        let start    = if selection.start.line > max_line { max_loc   } else { start };
//        let end      = selection.end;
//        let end      = if selection.end.line   < min_line { default() } else { end };
//        let end      = if selection.end.line   > max_line { max_loc   } else { end };
//        selection.with_start(start).with_end(end)
//    }

    /// Return the offset to the next grapheme if any. See the documentation of the library to
    /// learn more about graphemes.
    pub fn next_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.rope.next_grapheme_offset(offset.as_usize()).map(|t| Bytes(t as i32))
    }

    /// Return the offset to the previous grapheme if any. See the documentation of the library to
    /// learn more about graphemes.
    pub fn prev_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.rope.prev_grapheme_offset(offset.as_usize()).map(|t| Bytes(t as i32))
    }

    pub fn offset_of_line(&self, line:Line) -> Bytes {
        self.rope.offset_of_line(line.as_usize()).into()
    }

    pub fn line_of_offset(&self, offset:Bytes) -> Line {
        self.rope.line_of_offset(offset.as_usize()).into()
    }

    /// An iterator over the lines of a rope.
    ///
    /// Lines are ended with either Unix (`\n`) or MS-DOS (`\r\n`) style line endings.
    /// The line ending is stripped from the resulting string. The final line ending
    /// is optional.
    ///
    pub fn lines<T:rope::IntervalBounds>(&self, range:T) -> rope::Lines {
        self.rope.lines(range)
    }

}


// === Conversions ===

impl From<Rope>     for Text { fn from(t:Rope)     -> Self { Self {rope:t} } }
impl From<&Rope>    for Text { fn from(t:&Rope)    -> Self { t.clone().into() } }

impl From<&str>     for Text { fn from(t:&str)     -> Self { Self {rope:t.into()} } }
impl From<String>   for Text { fn from(t:String)   -> Self { Self {rope:t.into()} } }
impl From<&String>  for Text { fn from(t:&String)  -> Self { Self {rope:t.into()} } }
impl From<&&String> for Text { fn from(t:&&String) -> Self { (*t).into() } }
impl From<&&str>    for Text { fn from(t:&&str)    -> Self { (*t).into() } }

impl From<Text>     for String { fn from(t:Text)   -> Self { t.rope.into() } }
impl From<&Text>    for String { fn from(t:&Text)  -> Self { t.clone().into() } }
impl From<&&Text>   for String { fn from(t:&&Text) -> Self { (*t).into() } }
