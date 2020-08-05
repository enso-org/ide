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



// =========================
// === Generic Utilities ===
// =========================

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
            if let Some(off) = self.rope.next_grapheme_offset(offset) {
                offset = off;
                count += 1;
            } else { break }
        }
        count
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

    /// An iterator over the lines of a rope.
    ///
    /// Lines are ended with either Unix (`\n`) or MS-DOS (`\r\n`) style line endings. The line
    /// ending is stripped from the resulting string. The final line ending is optional.
    pub fn lines<T:rope::IntervalBounds>(&self, range:T) -> rope::Lines {
        self.rope.lines(range)
    }

    /// Replaces the provided range with the provided text.
    pub fn replace(&mut self, range:impl RangeBounds, text:&Text) {
        let range = self.clamp_byte_range(range);
        self.rope.edit(range.into_rope_interval(),text.rope.clone());
    }
}

// === First Line ===

impl Text {
    /// The first valid line index in this text.
    pub fn first_line_index(&self) -> Line {
        0.line()
    }

    /// The first valid line byte offset in this text.
    pub fn first_line_byte_offset(&self) -> Bytes {
        0.bytes()
    }

    /// The start column of the first line.
    pub fn first_line_start_column(&self) -> Column {
        0.column()
    }

    /// The start location of the first line.
    pub fn first_line_start_location(&self) -> Location {
        let line   = self.first_line_index();
        let column = self.first_line_start_column();
        Location(line,column)
    }
}


// === Last Line ===

impl Text {
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

    /// The start column of the last line.
    pub fn last_line_start_column(&self) -> Column {
        0.column()
    }

    /// The start location of the last line.
    pub fn last_line_start_location(&self) -> Location {
        let line   = self.last_line_index();
        let column = self.last_line_start_column();
        Location(line,column)
    }

    /// The last column number of the last line.
    pub fn last_line_end_column(&self) -> Column {
        self.column_from_byte_offset(self.byte_size()).unwrap()
    }

    /// The byte offset of the end of the last line. Equal to the byte size of the whole text.
    pub fn last_line_end_byte_offset(&self) -> Bytes {
        self.byte_size()
    }

    /// The location of the last character in the text.
    pub fn last_line_end_location(&self) -> Location {
        let line   = self.last_line_index();
        let column = self.last_line_end_column();
        Location(line,column)
    }
}


// === Validation ===

impl Text {
    /// Check whether the provided line index is valid in this text.
    pub fn validate_line_index(&self, line:Line) -> Result<(),LineIndexError> {
        use LineIndexError::*;
        if      line < 0.line()               {Err(LineIndexNegative)}
        else if line > self.last_line_index() {Err(LineIndexTooBig)}
        else                                  {Ok(())}
    }

    /// Check whether the provided byte offset is valid in this text.
    pub fn validate_byte_offset(&self, offset:Bytes) -> Result<(),ByteOffsetError> {
        use ByteOffsetError::*;
        if      offset < 0.bytes()        {Err(OffsetNegative)}
        else if offset > self.byte_size() {Err(OffsetTooBig)}
        else                              {Ok(())}
    }
}



// ===================
// === Conversions ===
// ===================

// === Into Byte Offset ===

impl Text {

    /// Return the offset after the last character of a given line if the line exists.
    pub fn end_byte_offset_from_line_index(&self, line:Line) -> Result<Bytes,LineIndexError> {
        self.validate_line_index(line)?;
        let next_line      = line + 1.line();
        let next_line_off  = self.byte_offset_from_line_index(next_line).ok();
        let next_line_prev = next_line_off.and_then(|t| self.prev_grapheme_offset(t));
        Ok(next_line_prev.unwrap_or_else(|| self.byte_size()))
    }

    /// The line byte offset. Panics in case the line index was invalid.
    pub fn byte_offset_from_line_index_unchecked(&self, line:Line) -> Bytes {
        self.rope.offset_of_line(line.as_usize()).into()
    }

    /// The byte offset of the given line index.
    pub fn byte_offset_from_line_index(&self, line:Line) -> Result<Bytes,LineIndexError> {
        self.validate_line_index(line)?;
        Ok(self.byte_offset_from_line_index_unchecked(line))
    }

    /// The byte offset of the given line. Snapped to the closest valid byte offset in case the
    /// line index was invalid.
    pub fn byte_offset_from_line_index_snapped(&self, line:Line) -> Bytes {
        use LineIndexError::*;
        match self.byte_offset_from_line_index(line) {
            Ok(offset)             => offset,
            Err(LineIndexNegative) => self.first_line_byte_offset(),
            Err(LineIndexTooBig)   => self.last_line_byte_offset(),
        }
    }

    /// Byte offset of the given location.
    pub fn byte_offset_from_location(&self, location:Location)
    -> Result<Bytes,LocationError<Bytes>> {
        let mut column = 0.column();
        let mut offset = self.byte_offset_from_line_index(location.line)?;
        let max_offset = self.end_byte_offset_from_line_index(location.line)?;
        while column < location.column {
            match self.next_grapheme_offset(offset) {
                None      => return Err(LocationError::LineTooShort(offset)),
                Some(off) => {
                    offset  = off;
                    column += 1.column();
                }
            }
        }
        if offset > max_offset {
            Err(LocationError::LineTooShort(max_offset))
        } else {
            Ok(offset)
        }
    }

    /// Byte offset of the given location. The result will be snapped to the closest valid value.
    pub fn byte_offset_from_location_snapped(&self, location:Location) -> Bytes {
        let offset = self.byte_offset_from_location(location);
        self.snap_bytes_location_result(offset)
    }
}


// === Into Line Index ===

impl Text {
    /// The line of a given byte offset. Panics in case the offset was invalid.
    pub fn line_index_from_byte_offset_unchecked(&self, offset:Bytes) -> Line {
        self.rope.line_of_offset(offset.as_usize()).into()
    }

    /// The line index of the given byte offset.
    pub fn line_index_from_byte_offset(&self, offset:Bytes) -> Result<Line,ByteOffsetError> {
        self.validate_byte_offset(offset)?;
        Ok(self.line_index_from_byte_offset_unchecked(offset))
    }

    /// The line index of the given byte offset. Snapped to the closest valid line index in case the
    /// byte offset was invalid.
    pub fn line_index_from_byte_offset_snapped(&self, offset:Bytes) -> Line {
        use ByteOffsetError::*;
        match self.line_index_from_byte_offset(offset) {
            Ok(index)           => index,
            Err(OffsetNegative) => self.first_line_index(),
            Err(OffsetTooBig)   => self.last_line_index(),
        }
    }
}


// === Into Column ===

impl Text {
    /// The last column number of the given line.
    pub fn line_end_column(&self, line:Line) -> Result<Column,LineIndexError> {
        let offset = self.end_byte_offset_from_line_index(line)?;
        Ok(self.column_from_byte_offset(offset).unwrap())
    }

    /// The column number of the given byte offset.
    pub fn column_from_byte_offset(&self, tgt_offset:Bytes)
    -> Result<Column,ColumnFromByteOffsetError> {
        use ColumnFromByteOffsetError::*;
        let line_index = self.line_index_from_byte_offset(tgt_offset)?;
        let mut offset = self.byte_offset_from_line_index(line_index)?;
        let mut column = 0.column();
        while offset < tgt_offset {
            match self.next_grapheme_offset(offset) {
                None      => return Err(OffsetTooBig),
                Some(off) => {
                    offset  = off;
                    column += 1.column();
                }
            }
        }
        if offset != tgt_offset {
            Err(NotClusterBoundary(column))
        } else {
            Ok(column)
        }
    }

    /// The column number of the given byte offset. The result will be snapped to the closest valid
    /// value. In case the offset points inside of a grapheme cluster, it will be snapped to its
    /// right side.
    pub fn column_from_byte_offset_snapped(&self, tgt_offset:Bytes) -> Column {
        use ColumnFromByteOffsetError::*;
        match self.column_from_byte_offset(tgt_offset) {
            Ok(column)                      => column,
            Err(OffsetNegative)             => 0.column(),
            Err(OffsetTooBig)               => self.last_line_end_column(),
            Err(NotClusterBoundary(column)) => column,
        }
    }

    /// The column from line number and byte offset within the line.
    pub fn column_from_line_index_and_in_line_byte_offset(&self, line:Line, in_line_offset:Bytes)
    -> Result<Column,LocationError<Column>> {
        use LocationError::*;
        let mut offset = self.byte_offset_from_line_index(line)?;
        let tgt_offset = offset + in_line_offset;
        let column     = self.column_from_byte_offset(tgt_offset)?;
        Ok(column)
    }

    /// The column from line number and byte offset within the line. The result will be snapped to
    /// the closest valid value. In case the offset points inside of a grapheme cluster, it will be
    /// snapped to its right side.
    pub fn column_from_line_index_and_in_line_byte_offset_snapped
    (&self, line:Line, in_line_offset:Bytes) -> Column {
        let column = self.column_from_line_index_and_in_line_byte_offset(line,in_line_offset);
        self.snap_column_location_result(column)
    }
}


// === Into Location ===

impl Text {
    /// The location of the provided byte offset.
    pub fn location_from_byte_offset(&self, offset:Bytes) -> Result<Location,ByteOffsetError> {
        let line        = self.line_index_from_byte_offset(offset)?;
        let line_offset = (offset - self.byte_offset_from_line_index(line).unwrap());
        let column      = self.column_from_line_index_and_in_line_byte_offset(line,line_offset);
        let column      = column.unwrap();
        Ok(Location(line,column))
    }

    /// The location of the provided byte offset. The result will be snapped to the closest valid
    /// value.
    pub fn location_from_byte_offset_snapped(&self, offset:Bytes) -> Location {
        use ByteOffsetError::*;
        match self.location_from_byte_offset(offset) {
            Ok(location)        => location,
            Err(OffsetNegative) => self.first_line_start_location(),
            Err(OffsetTooBig)   => self.last_line_end_location(),
        }
    }
}



// ==============
// === Errors ===
// ==============

#[derive(Clone,Copy,Debug)]
pub enum LineIndexError {
    LineIndexNegative,
    LineIndexTooBig,
}

#[derive(Clone,Copy,Debug)]
pub enum ByteOffsetError {
    OffsetNegative,
    OffsetTooBig,
}

#[derive(Clone,Copy,Debug)]
pub enum LocationError<T> {
    LineIndexNegative,
    LineIndexTooBig,
    LineTooShort       (T),
    NotClusterBoundary (T)
}

#[derive(Clone,Copy,Debug)]
pub enum ColumnFromByteOffsetError {
    OffsetNegative,
    OffsetTooBig,
    NotClusterBoundary (Column)
}

impl From<ColumnFromByteOffsetError> for LocationError<Column> {
    fn from(err:ColumnFromByteOffsetError) -> Self {
        use LocationError::*;
        match err {
            ColumnFromByteOffsetError::OffsetNegative        => LineIndexNegative,
            ColumnFromByteOffsetError::OffsetTooBig          => LineIndexTooBig,
            ColumnFromByteOffsetError::NotClusterBoundary(t) => NotClusterBoundary(t),
        }
    }
}

impl<T> From<LineIndexError> for LocationError<T> {
    fn from(err:LineIndexError) -> Self {
        use LocationError::*;
        match err {
            LineIndexError::LineIndexNegative => LineIndexNegative,
            LineIndexError::LineIndexTooBig   => LineIndexTooBig,
        }
    }
}

impl From<ByteOffsetError> for ColumnFromByteOffsetError {
    fn from(err:ByteOffsetError) -> Self {
        match err {
            ByteOffsetError::OffsetNegative => ColumnFromByteOffsetError::OffsetNegative,
            ByteOffsetError::OffsetTooBig   => ColumnFromByteOffsetError::OffsetTooBig,
        }
    }
}

impl From<LineIndexError> for ColumnFromByteOffsetError {
    fn from(err:LineIndexError) -> Self {
        match err {
            LineIndexError::LineIndexNegative => ColumnFromByteOffsetError::OffsetNegative,
            LineIndexError::LineIndexTooBig   => ColumnFromByteOffsetError::OffsetTooBig,
        }
    }
}

impl Text {
    /// Snaps the `LocationError<Column>` to the closest valid column.
    pub fn snap_column_location_error(&self, err:LocationError<Column>) -> Column {
        use LocationError::*;
        match err {
            LineIndexNegative           => 0.column(),
            LineIndexTooBig             => self.last_line_end_column(),
            LineTooShort       (column) => column,
            NotClusterBoundary (column) => column,
        }
    }

    /// Snaps the `LocationError<Bytes>` to the closest valid byte offset.
    pub fn snap_bytes_location_error(&self, err:LocationError<Bytes>) -> Bytes {
        use LocationError::*;
        match err {
            LineIndexNegative           => 0.bytes(),
            LineIndexTooBig             => self.last_line_end_byte_offset(),
            LineTooShort       (offset) => offset,
            NotClusterBoundary (offset) => offset,
        }
    }

    /// Snaps the `LocationResult<Column>` to the closest valid column.
    pub fn snap_column_location_result
    (&self, result:Result<Column,LocationError<Column>>) -> Column {
        match result {
            Ok(column) => column,
            Err(err)   => self.snap_column_location_error(err),
        }
    }

    /// Snaps the `LocationResult<Bytes>` to the closest valid byte offset.
    pub fn snap_bytes_location_result
    (&self, result:Result<Bytes,LocationError<Bytes>>) -> Bytes {
        match result {
            Ok(bytes) => bytes,
            Err(err)  => self.snap_bytes_location_error(err),
        }
    }
}



// ===================
// === Conversions ===
// ===================

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



// ================
// === TextCell ===
// ================

/// Internally mutable version of `Text`.
#[derive(Debug,Clone,Default,Deref)]
#[allow(missing_docs)]
pub struct TextCell {
    pub cell : RefCell<Text>
}

/// See docs in `Text`.
#[allow(missing_docs)]
impl TextCell {
    pub fn new() -> Self {
        default()
    }

    pub fn is_empty(&self) -> bool {
        self.cell.borrow().is_empty()
    }

    pub fn grapheme_count(&self) -> usize {
        self.cell.borrow().grapheme_count()
    }

    pub fn byte_size(&self) -> Bytes {
        self.cell.borrow().byte_size()
    }

    pub fn byte_range(&self) -> Range<Bytes> {
        self.cell.borrow().byte_range()
    }

    pub fn clamp_byte_range(&self, range:impl RangeBounds) -> Range<Bytes> {
        self.cell.borrow().clamp_byte_range(range)
    }

    pub fn next_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.cell.borrow().next_grapheme_offset(offset)
    }

    pub fn prev_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.cell.borrow().prev_grapheme_offset(offset)
    }

//    pub fn lines<T:rope::IntervalBounds>(&self, range:T) -> rope::Lines {
//        self.cell.borrow().lines(range)
//    }

    pub fn replace(&self, range:impl RangeBounds, text:&Text) {
        self.cell.borrow_mut().replace(range,text)
    }

    pub fn first_line_index(&self) -> Line {
        self.cell.borrow().first_line_index()
    }

    pub fn first_line_byte_offset(&self) -> Bytes {
        self.cell.borrow().first_line_byte_offset()
    }

    pub fn first_line_start_column(&self) -> Column {
        self.cell.borrow().first_line_start_column()
    }

    pub fn first_line_start_location(&self) -> Location {
        self.cell.borrow().first_line_start_location()
    }

    pub fn last_line_index(&self) -> Line {
        self.cell.borrow().last_line_index()
    }

    pub fn last_line_byte_offset(&self) -> Bytes {
        self.cell.borrow().last_line_byte_offset()
    }

    pub fn last_line_start_column(&self) -> Column {
        self.cell.borrow().last_line_start_column()
    }

    pub fn last_line_start_location(&self) -> Location {
        self.cell.borrow().last_line_start_location()
    }

    pub fn last_line_end_column(&self) -> Column {
        self.cell.borrow().last_line_end_column()
    }

    pub fn last_line_end_byte_offset(&self) -> Bytes {
        self.cell.borrow().last_line_end_byte_offset()
    }

    pub fn last_line_end_location(&self) -> Location {
        self.cell.borrow().last_line_end_location()
    }

    pub fn validate_line_index(&self, line:Line) -> Result<(),LineIndexError> {
        self.cell.borrow().validate_line_index(line)
    }

    pub fn validate_byte_offset(&self, offset:Bytes) -> Result<(),ByteOffsetError> {
        self.cell.borrow().validate_byte_offset(offset)
    }

    pub fn end_byte_offset_from_line_index(&self, line:Line) -> Result<Bytes,LineIndexError> {
        self.cell.borrow().end_byte_offset_from_line_index(line)
    }

    pub fn byte_offset_from_line_index_unchecked(&self, line:Line) -> Bytes {
        self.cell.borrow().byte_offset_from_line_index_unchecked(line)
    }

    pub fn byte_offset_from_line_index(&self, line:Line) -> Result<Bytes,LineIndexError> {
        self.cell.borrow().byte_offset_from_line_index(line)
    }

    pub fn byte_offset_from_line_index_snapped(&self, line:Line) -> Bytes {
        self.cell.borrow().byte_offset_from_line_index_snapped(line)
    }

    pub fn byte_offset_from_location(&self, location:Location)
    -> Result<Bytes,LocationError<Bytes>> {
        self.cell.borrow().byte_offset_from_location(location)
    }

    pub fn byte_offset_from_location_snapped(&self, location:Location) -> Bytes {
        self.cell.borrow().byte_offset_from_location_snapped(location)
    }

    pub fn line_index_from_byte_offset_unchecked(&self, offset:Bytes) -> Line {
        self.cell.borrow().line_index_from_byte_offset_unchecked(offset)
    }

    pub fn line_index_from_byte_offset(&self, offset:Bytes) -> Result<Line,ByteOffsetError> {
        self.cell.borrow().line_index_from_byte_offset(offset)
    }

    pub fn line_index_from_byte_offset_snapped(&self, offset:Bytes) -> Line {
        self.cell.borrow().line_index_from_byte_offset_snapped(offset)
    }

    pub fn line_end_column(&self, line:Line) -> Result<Column,LineIndexError> {
        self.cell.borrow().line_end_column(line)
    }

    pub fn column_from_byte_offset(&self, tgt_offset:Bytes)
    -> Result<Column,ColumnFromByteOffsetError> {
        self.cell.borrow().column_from_byte_offset(tgt_offset)
    }

    pub fn column_from_byte_offset_snapped(&self, tgt_offset:Bytes) -> Column {
        self.cell.borrow().column_from_byte_offset_snapped(tgt_offset)
    }

    pub fn column_from_line_index_and_in_line_byte_offset(&self, line:Line, in_line_offset:Bytes)
    -> Result<Column,LocationError<Column>> {
        self.cell.borrow().column_from_line_index_and_in_line_byte_offset(line,in_line_offset)
    }

    pub fn column_from_line_index_and_in_line_byte_offset_snapped
    (&self, line:Line, in_line_offset:Bytes) -> Column {
        self.cell.borrow().column_from_line_index_and_in_line_byte_offset_snapped
            (line,in_line_offset)
    }

    pub fn location_from_byte_offset(&self, offset:Bytes) -> Result<Location,ByteOffsetError> {
        self.cell.borrow().location_from_byte_offset(offset)
    }

    pub fn location_from_byte_offset_snapped(&self, offset:Bytes) -> Location {
        self.cell.borrow().location_from_byte_offset_snapped(offset)
    }
}
