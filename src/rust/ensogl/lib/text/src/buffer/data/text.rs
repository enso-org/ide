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


// === Constructors and Info ===

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

    /// Return the len of the text in bytes.
    pub fn byte_size(&self) -> Bytes {
        Bytes(self.rope.len() as i32)
    }

    /// Range of the text in bytes.
    pub fn byte_range(&self) -> Range<Bytes> {
        (..self.byte_size()).into()
    }

    /// Constraint the provided byte range so it will be contained of the range of this data. This
    /// ensures that the provided range will be valid for operations on this data.
    pub fn clamp_byte_range(&self, range:impl RangeBounds) -> Range<Bytes> {
        range.with_upper_bound(self.byte_size())
    }

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

    pub fn line_of_offset(&self, offset:Bytes) -> Line {
        self.rope.line_of_offset(offset.as_usize()).into()
    }

    /// An iterator over the lines of a rope.
    ///
    /// Lines are ended with either Unix (`\n`) or MS-DOS (`\r\n`) style line endings. The line
    /// ending is stripped from the resulting string. The final line ending is optional.
    pub fn lines<T:rope::IntervalBounds>(&self, range:T) -> rope::Lines {
        println!("---------");
        println!("last_line_index: {:?}",self.last_line_index());
        println!("last_line_byte_offset: {:?}",self.last_line_byte_offset());
        println!("byte_size(): {:?}",self.byte_size());
        self.rope.lines(range)
    }

}



// === Line Indexing ===

#[derive(Clone,Copy,Debug)]
pub enum ByteOffsetFromLineIndexError {
    LineIndexNegative,
    LineIndexTooBig,
}

#[derive(Clone,Copy,Debug)]
pub enum LineIndexFromByteOffsetError {
    OffsetNegative,
    OffsetTooBig,
}

impl Text {
    /// The first valid line index in this text.
    pub fn first_line_index(&self) -> Line {
        0.line()
    }

    /// The first valid line byte offset in this text.
    pub fn first_line_byte_offset(&self) -> Bytes {
        0.bytes()
    }

    /// The last valid line index in this text. If the text ends with the newline character,
    /// it means that there is an empty last line.
    pub fn last_line_index(&self) -> Line {
        (self.rope.measure::<rope::metric::Lines>()).into()
    }

    /// The last valid line byte offset in this text. If the text ends with the newline character,
    /// it means that there is an empty last line.
    pub fn last_line_byte_offset(&self) -> Bytes {
        self.byte_offset_from_line_index_unchecked(self.last_line_index())
    }

    /// The line byte offset. Panics in case the line index was invalid.
    pub fn byte_offset_from_line_index_unchecked(&self, line:Line) -> Bytes {
        self.rope.offset_of_line(line.as_usize()).into()
    }

    /// The line of a given byte offset. Panics in case the offset was invalid.
    pub fn line_from_byte_offset_unchecked(&self, offset:Bytes) -> Line {
        self.rope.line_of_offset(offset.as_usize()).into()
    }

    /// The byte offset of the given line index.
    pub fn byte_offset_from_line_index(&self, line:Line)
    -> Result<Bytes,ByteOffsetFromLineIndexError> {
        use ByteOffsetFromLineIndexError::*;
        if      line < 0.line()               {Err(LineIndexNegative)}
        else if line > self.last_line_index() {Err(LineIndexTooBig)}
        else                                  {Ok(self.byte_offset_from_line_index_unchecked(line))}
    }

    /// The line index of the given byte offset.
    pub fn line_index_from_byte_offset(&self, offset:Bytes)
    -> Result<Line,LineIndexFromByteOffsetError> {
        use LineIndexFromByteOffsetError::*;
        if      offset < 0.bytes()        {Err(OffsetNegative)}
        else if offset > self.byte_size() {Err(OffsetTooBig)}
        else                              {Ok(self.line_from_byte_offset_unchecked(offset))}
    }

    /// The byte offset of the given line. Snapped to the closest valid byte offset in case the
    /// line index was invalid.
    pub fn byte_offset_from_line_index_snapped(&self, line:Line) -> Bytes {
        use ByteOffsetFromLineIndexError::*;
        match self.byte_offset_from_line_index(line) {
            Ok(offset)             => offset,
            Err(LineIndexNegative) => self.first_line_byte_offset(),
            Err(LineIndexTooBig)   => self.last_line_byte_offset(),
        }
    }

    /// The line index of the given byte offset. Snapped to the closest valid line index in case the
    /// byte offset was invalid.
    pub fn line_index_from_byte_offset_snapped(&self, offset:Bytes) -> Line {
        use LineIndexFromByteOffsetError::*;
        match self.line_index_from_byte_offset(offset) {
            Ok(index)           => index,
            Err(OffsetNegative) => self.first_line_index(),
            Err(OffsetTooBig)   => self.last_line_index(),
        }
    }
}


// === Column Indexing ===

pub enum ColumnFromLineAndOffsetError {
    LineIndexNegative,
    LineIndexTooBig,
    LineTooShort,
    NotClusterBoundary(Bytes,Bytes)
}

impl Text {
//    /// Compute the column based on line number and byte offset within the line. The column will
//    /// be snapped to the right side of the grapheme cluster in case the offset will point inside
//    /// of a cluster. Returns `None` if matching was impossible, which can happen if the line number
//    /// was negative, bigger then total line number, or the line offset was bigger than line length.


//    pub fn column_from_line_and_offset(&self, line:Line, line_offset:Bytes) -> Option<Column> {
//        let mut offset = self.byte_offset_from_line_index(line).ok()?;
//        let tgt_offset = offset + line_offset;
//        let mut column = 0.column();
//        while offset < tgt_offset {
//            match self.next_grapheme_offset(offset) {
//                None => return None,
//                Some(off) => {
//                    column += 1.column();
//                    offset = off;
//                }
//            }
//        }
//        Some(column)
//    }

    pub fn column_from_line_and_offset(&self, line:Line, line_offset:Bytes) -> Option<Column> {
        let mut offset = self.byte_offset_from_line_index(line).ok()?;
        let tgt_offset = offset + line_offset;
        let mut column = 0.column();
        while offset < tgt_offset {
            match self.next_grapheme_offset(offset) {
                None => return None,
                Some(off) => {
                    column += 1.column();
                    offset = off;
                }
            }
        }
        Some(column)
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
