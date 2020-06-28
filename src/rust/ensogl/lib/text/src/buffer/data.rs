//! The data hold by the text buffer. Under the hood it is implemented as an efficient string rope.

use crate::prelude::*;



// ===============
// === Exports ===
// ===============

pub mod range;
pub mod rope;
pub mod spans;
pub mod unit;

pub use range::Range;
pub use range::RangeBounds;
pub use rope::Delta;
pub use rope::Rope;
pub use rope::Cursor;
pub use rope::Lines;
pub use rope::LinesMetric;
pub use spans::*;
pub use unit::*;



// ============
// === Data ===
// ============

impl_clone_ref_as_clone!(Data);
#[derive(Debug,Clone,Default,Deref)]
#[allow(missing_docs)]
pub struct Data {
    pub rope : Rope,
}

impl Data {
    /// Return the len of the text in bytes.
    pub fn len(&self) -> Bytes {
        Bytes(self.rope.len())
    }

    /// Check whether the text is empty.
    pub fn is_empty(&self) -> bool {
        self.rope.is_empty()
    }

    /// Range of the text in this data.
    pub fn range(&self) -> Range<Bytes> {
        (..self.len()).into()
    }

    /// Crop the provided range so it will be contained of the range of this data. This ensures that
    /// the provided range will be valid for operations on this data.
    pub fn crop_range(&self, range:impl RangeBounds) -> Range<Bytes> {
        range.with_upper_bound(self.len())
    }

    /// Return the offset to the previous grapheme if any.
    pub fn prev_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.rope.prev_grapheme_offset(offset.raw).map(Bytes)
    }

    /// Return the offset to the next grapheme if any.
    pub fn next_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.rope.next_grapheme_offset(offset.raw).map(Bytes)
    }
}


// === Constructors ===

impl From<Rope>     for Data { fn from(t:Rope)     -> Self { Self {rope:t} } }
impl From<&Rope>    for Data { fn from(t:&Rope)    -> Self { t.clone().into() } }

impl From<&str>     for Data { fn from(t:&str)     -> Self { Self {rope:t.into()} } }
impl From<String>   for Data { fn from(t:String)   -> Self { Self {rope:t.into()} } }
impl From<&String>  for Data { fn from(t:&String)  -> Self { Self {rope:t.into()} } }
impl From<&&String> for Data { fn from(t:&&String) -> Self { (*t).into() } }
impl From<&&str>    for Data { fn from(t:&&str)    -> Self { (*t).into() } }
