#![allow(missing_docs)]

//! Root of text buffer implementation. The text buffer is a sophisticated model for text styling
//! and editing operations.

use crate::prelude::*;



// ===============
// === Exports ===
// ===============

pub mod data;
pub mod style;
pub mod view;

/// Common traits.
pub mod traits {
    pub use super::data::traits::*;
    pub use super::Setter        as TRAIT_Setter;
    pub use super::DefaultSetter as TRAIT_DefaultSetter;
}

pub use data::Text;
pub use data::TextCell;
pub use data::text::LineIndexError;
pub use data::Range;
pub use data::unit::*;
pub use view::*;
pub use style::*;



// fixme - refactor to undo/redo stub
// ================
// === EditType ===
// ================

#[derive(PartialEq,Eq,Clone,Copy,Debug)]
pub enum EditType {
    /// A catchall for edits that don't fit elsewhere, and which should
    /// always have their own undo groups; used for things like cut/copy/paste.
    Other,
    /// An insert from the keyboard/IME (not a paste or a yank).
    Insert,
    Newline,
    /// An indentation adjustment.
    Indent,
    Delete,
    Undo,
    Redo,
    Transpose,
    Surround,
}

impl EditType {
    // /// Checks whether a new undo group should be created between two edits.
    // fn breaks_undo_group(self, previous:EditType) -> bool {
    //     self == EditType::Other || self == EditType::Transpose || self != previous
    // }
}

impl Default for EditType {
    fn default() -> Self {
        Self::Other
    }
}



// ==============
// === Buffer ===
// ==============

#[derive(Clone,CloneRef,Debug,Default)]
pub struct Buffer {
    pub(crate) data : Rc<RefCell<BufferData>>
}

impl Buffer {
    pub fn new() -> Self {
        default()
    }

    pub fn borrow(&self) -> Ref<'_,BufferData> {
        self.data.borrow()
    }

//    pub fn borrow_mut(&self) -> RefMut<'_,BufferData> {
//        self.data.borrow_mut()
//    }

    pub fn data(&self) -> Text {
        self.data.borrow().text.clone()
    }

    pub fn last_line_index(&self) -> Line {
        self.data.borrow().last_line_index()
    }

    pub fn line_index_from_byte_offset_snapped(&self, offset:Bytes) -> Line {
        self.data.borrow().line_index_from_byte_offset_snapped(offset)
    }

    fn byte_offset_from_line_index(&self,line:Line) -> Result<Bytes,LineIndexError> {
        self.data.borrow().byte_offset_from_line_index(line)
    }

    pub fn end_byte_offset_from_line_index(&self, line:Line) -> Result<Bytes,LineIndexError> {
        self.data.borrow().end_byte_offset_from_line_index(line)
    }

    pub fn byte_offset_from_location_snapped(&self, location:Location) -> Bytes {
        self.data.borrow().byte_offset_from_location_snapped(location)
    }

    fn last_offset(&self) -> Bytes {
        self.data().byte_size()
    }

    pub fn style(&self) -> Style {
        self.data.borrow().style()
    }

    pub fn set_data(&self, text:Text) {
        self.data.borrow_mut().text = text;
    }

    pub fn set_style(&self, style:Style) {
        *self.data.borrow().style.borrow_mut() = style;
    }

    /// Creates a new `View` for the buffer.
    pub fn new_view(&self) -> View {
        View::new(self)
    }

    pub fn sub_style(&self, range:impl data::RangeBounds) -> Style {
        self.data.borrow().sub_style(range)
    }

    pub fn column_from_line_index_and_in_line_byte_offset_snapped
    (&self, line:Line, line_offset:Bytes) -> Column {
        self.data.borrow().column_from_line_index_and_in_line_byte_offset_snapped(line,line_offset)
    }

    /// Return the offset to the next grapheme if any. See the documentation of the library to
    /// learn more about graphemes.
    pub fn next_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.data.borrow().text.next_grapheme_offset(offset)
    }

    /// Return the offset to the previous grapheme if any. See the documentation of the library to
    /// learn more about graphemes.
    pub fn prev_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.data.borrow().text.prev_grapheme_offset(offset)
    }
}



// ==================
// === BufferData ===
// ==================

/// Text container with associated styles.
#[derive(Debug,Default)]
pub struct BufferData {
    pub(crate) text  : Text,
    pub(crate) style : Rc<RefCell<Style>>,
}

impl Deref for BufferData {
    type Target = Text;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl BufferData {
    pub fn new() -> Self {
        default()
    }

    pub fn sub_style(&self, range:impl data::RangeBounds) -> Style {
        let range = self.clamp_byte_range(range);
        self.style.borrow().sub(range)
    }

    pub fn style(&self) -> Style {
        self.style.borrow().clone()
    }

    pub fn insert(&mut self, range:impl data::RangeBounds, text:&Text) {
        let range = self.clamp_byte_range(range);
        self.text.rope.edit(range.into_rope_interval(),text.rope.clone());
        self.style.borrow_mut().modify(range,text.byte_size());
    }
}



// ==============
// === Setter ===
// ==============

pub trait Setter<T> {
    fn modify(&self, range:impl data::RangeBounds, len:Bytes, data:T);
    fn set(&self, range:impl data::RangeBounds, data:T);
}

pub trait DefaultSetter<T> {
    fn set_default(&self, data:T);
}
