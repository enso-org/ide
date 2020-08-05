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



// ==============
// === Buffer ===
// ==============

/// Internally mutable text container with associated styles.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Buffer {
    data : Rc<BufferData>
}

impl Deref for Buffer {
    type Target = BufferData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl Buffer {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Creates a new `View` for the buffer.
    pub fn new_view(&self) -> View {
        View::new(self)
    }
}



// ==================
// === BufferData ===
// ==================

/// Internal data of `Buffer`.
#[derive(Debug,Default)]
pub struct BufferData {
    pub(crate) text  : TextCell,
    pub(crate) style : RefCell<Style>,
}

impl Deref for BufferData {
    type Target = TextCell;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl BufferData {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Text clone. Note that this is a fast clone, as text is immutable.
    pub fn text(&self) -> Text {
        self.text.cell.borrow().clone()
    }

    /// Style clone.
    pub fn style(&self) -> Style {
        self.style.borrow().clone()
    }

    /// Sets the new text content.
    pub fn set_text(&self, text:impl Into<Text>) {
        self.text.set(text);
    }

    /// Set new styles.
    pub fn set_style(&self, style:Style) {
        *self.style.borrow_mut() = style;
    }

    /// Query style information for the provided range.
    pub fn sub_style(&self, range:impl data::RangeBounds) -> Style {
        let range = self.clamp_byte_range(range);
        self.style.borrow().sub(range)
    }

    /// Replaces the provided range with the provided text. The style will be set to default within
    /// the range.
    pub fn replace(&self, range:impl data::RangeBounds, text:&Text) {
        let range = self.clamp_byte_range(range);
        self.text.replace(range,text);
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
