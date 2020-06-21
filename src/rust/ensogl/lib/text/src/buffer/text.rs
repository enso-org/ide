
pub mod rope {
    pub use xi_rope::*;
    pub use xi_rope::rope::Lines;
    pub use xi_rope::spans::Spans;
    pub use xi_rope::spans::SpansBuilder;
    pub use xi_rope::spans::SpansInfo;
    pub use xi_rope::interval::Interval;
}

pub use rope::SpansBuilder;
pub use rope::Cursor;
pub use rope::LinesMetric;
pub use rope::Lines;

use crate::prelude::*;
use crate::buffer::location::*;

use rope::Rope;



// ============
// === Text ===
// ============

impl_clone_ref_as_clone!(Text);
#[derive(Debug,Clone,Default,Deref)]
#[allow(missing_docs)]
pub struct Text {
    pub rope : Rope,
}

impl Text {
    /// Return the len of the text in bytes.
    pub fn len(&self) -> Bytes {
        Bytes(self.rope.len())
    }

    pub fn range(&self) -> Range<Bytes> {
        (..self.len()).into()
    }

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

impl From<&str>     for Text { fn from(t:&str)     -> Self { Self {rope:t.into()} } }
impl From<String>   for Text { fn from(t:String)   -> Self { Self {rope:t.into()} } }
impl From<&String>  for Text { fn from(t:&String)  -> Self { Self {rope:t.into()} } }
impl From<&&String> for Text { fn from(t:&&String) -> Self { (*t).into() } }
impl From<&&str>    for Text { fn from(t:&&str)    -> Self { (*t).into() } }



// =============
// === Range ===
// =============

#[derive(Clone,Copy,PartialEq,Eq)]
pub struct Range<T> {
    pub start : T,
    pub end   : T,
}

impl<T> Range<T> {
    pub fn new(start:T, end:T) -> Self {
        Self {start,end}
    }

    pub fn size(&self) -> T where T : Clone + Sub<T,Output=T> {
        self.end.clone() - self.start.clone()
    }
}

impl Range<Bytes> {
    pub fn into_rope_repr(self) -> rope::Interval {
        self.into()
    }
}

impl fmt::Display for Range<Bytes> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {})", self.start.raw, self.end.raw)
    }
}

impl fmt::Debug for Range<Bytes> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl From<std::ops::Range<Bytes>> for Range<Bytes> {
    fn from(range:std::ops::Range<Bytes>) -> Range<Bytes> {
        let std::ops::Range {start,end} = range;
        Range {start,end}
    }
}

impl From<std::ops::RangeTo<Bytes>> for Range<Bytes> {
    fn from(range:std::ops::RangeTo<Bytes>) -> Range<Bytes> {
        Range::new(Bytes(0), range.end)
    }
}

impl From<std::ops::RangeInclusive<Bytes>> for Range<Bytes> {
    fn from(range:std::ops::RangeInclusive<Bytes>) -> Range<Bytes> {
        Range::new(*range.start(), range.end().saturating_add(1))
    }
}

impl From<std::ops::RangeToInclusive<Bytes>> for Range<Bytes> {
    fn from(range:std::ops::RangeToInclusive<Bytes>) -> Range<Bytes> {
        Range::new(Bytes(0), range.end.saturating_add(1))
    }
}


// === Conversions ===

impl From<Range<Bytes>> for rope::Interval {
    fn from(t:Range<Bytes>) -> Self {
        let start = t.start.raw;
        let end   = t.end.raw;
        Self {start,end}
    }
}



// ======================
// === RangeBounds ===
// ======================

pub trait RangeBounds {
    fn with_upper_bound(self, upper_bound:Bytes) -> Range<Bytes>;
}

impl<T: Into<Range<Bytes>>> RangeBounds for T {
    fn with_upper_bound(self, _upper_bound:Bytes) -> Range<Bytes> {
        self.into()
    }
}

impl RangeBounds for RangeFrom<Bytes> {
    fn with_upper_bound(self, upper_bound:Bytes) -> Range<Bytes> {
        Range::new(self.start, upper_bound)
    }
}

impl RangeBounds for RangeFull {
    fn with_upper_bound(self, upper_bound:Bytes) -> Range<Bytes> {
        Range::new(Bytes(0),upper_bound)
    }
}



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
