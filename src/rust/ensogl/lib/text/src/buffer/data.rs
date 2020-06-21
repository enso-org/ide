//! The data hold by the text buffer. Under the hood it is implemented as an efficient string rope.

use crate::prelude::*;
use rope::Rope;



// ===============
// === Exports ===
// ===============

pub mod range;
pub mod unit;

pub mod rope {
    pub use xi_rope::*;
    pub use xi_rope::interval::Interval;
    pub use xi_rope::rope::Lines;
    pub use xi_rope::spans::Spans;
    pub use xi_rope::spans::SpansBuilder;
    pub use xi_rope::spans::SpansInfo;
}

pub use range::Range;
pub use range::RangeBounds;
pub use rope::Cursor;
pub use rope::Lines;
pub use rope::LinesMetric;
pub use rope::SpansBuilder;
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

impl From<&str>     for Data { fn from(t:&str)     -> Self { Self {rope:t.into()} } }
impl From<String>   for Data { fn from(t:String)   -> Self { Self {rope:t.into()} } }
impl From<&String>  for Data { fn from(t:&String)  -> Self { Self {rope:t.into()} } }
impl From<&&String> for Data { fn from(t:&&String) -> Self { (*t).into() } }
impl From<&&str>    for Data { fn from(t:&&str)    -> Self { (*t).into() } }







// =============
// === Spans ===
// =============

#[derive(Clone,Debug,Default)]
pub struct Spans<T:Clone> {
    raw : rope::Spans<Option<T>>
}

impl<T:Clone> Spans<T> {
    pub fn len(&self) -> Bytes {
        Bytes(self.raw.len())
    }

    pub fn set(&mut self, range:Range<Bytes>, data:Option<T>) {
        let mut builder = SpansBuilder::new(range.size().raw);
        builder.add_span((..),data);
        self.edit(range,builder.build());
    }

    // FIXME: remove as soon as we have editing ops
    pub fn TMP_set_default(&mut self, range:Range<Bytes>) {
        self.set(range,None);
    }

//    pub fn subseq(&self, bounds:impl rope::RangeBounds) -> rope::tree::Node<rope::SpansInfo<T>> {
//        self.rc.borrow().subseq(bounds)
//    }

    pub fn focus(&self, range:Range<Bytes>) -> Self {
        let raw = self.raw.subseq(range.into_rope_repr());
        Self {raw}
    }

    pub fn to_vector(&self) -> Vec<(Range<Bytes>,Option<T>)> {
        self.raw.iter().map(|t| (Range::new(Bytes(t.0.start),Bytes(t.0.end)),t.1.clone())).collect_vec()
    }

    pub fn edit
    (&mut self, range:Range<Bytes>, new:impl Into<rope::tree::Node<rope::SpansInfo<Option<T>>>>) {
        self.raw.edit(range.into_rope_repr(),new)
    }

    pub fn raw(&self) -> &rope::Spans<Option<T>> {
        &self.raw
    }

    pub fn raw_mut(&mut self) -> &mut rope::Spans<Option<T>> {
        &mut self.raw
    }
}
