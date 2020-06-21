
pub mod rope {
    pub use xi_rope::*;
    pub use xi_rope::rope::Lines;
    pub use xi_rope::spans::Spans;
    pub use xi_rope::spans::SpansBuilder;
    pub use xi_rope::spans::SpansInfo;
    pub use xi_rope::interval::IntervalBounds;
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



// ================
// === Interval ===
// ================

#[derive(Clone,Copy,PartialEq,Eq)]
pub struct Interval {
    pub start : Bytes,
    pub end   : Bytes,
}

impl Interval {
    pub fn new(start:Bytes, end:Bytes) -> Self {
        Self {start,end}
    }

    pub fn size(&self) -> Bytes {
        self.end - self.start
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {})", self.start.raw, self.end.raw)
    }
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl From<Range<Bytes>> for Interval {
    fn from(src:Range<Bytes>) -> Interval {
        let Range {start,end} = src;
        Interval {start,end}
    }
}

impl From<RangeTo<Bytes>> for Interval {
    fn from(src: RangeTo<Bytes>) -> Interval {
        Interval::new(Bytes(0), src.end)
    }
}

impl From<RangeInclusive<Bytes>> for Interval {
    fn from(src: RangeInclusive<Bytes>) -> Interval {
        Interval::new(*src.start(), src.end().saturating_add(1))
    }
}

impl From<RangeToInclusive<Bytes>> for Interval {
    fn from(src: RangeToInclusive<Bytes>) -> Interval {
        Interval::new(Bytes(0), src.end.saturating_add(1))
    }
}


// === Conversions ===

impl From<Interval> for rope::Interval {
    fn from(t:Interval) -> Self {
        let start = t.start.raw;
        let end   = t.end.raw;
        Self {start,end}
    }
}



// ======================
// === IntervalBounds ===
// ======================

pub trait IntervalBounds {
    fn with_upper_bound(self, upper_bound:Bytes) -> Interval;
}

impl<T: Into<Interval>> IntervalBounds for T {
    fn with_upper_bound(self, _upper_bound:Bytes) -> Interval {
        self.into()
    }
}

impl IntervalBounds for RangeFrom<Bytes> {
    fn with_upper_bound(self, upper_bound:Bytes) -> Interval {
        Interval::new(self.start, upper_bound)
    }
}

impl IntervalBounds for RangeFull {
    fn with_upper_bound(self, upper_bound:Bytes) -> Interval {
        Interval::new(Bytes(0),upper_bound)
    }
}



// =============
// === Spans ===
// =============

#[derive(CloneRef,Clone,Debug,Default)]
pub struct Spans<T:Clone> {
    rc : Rc<RefCell<rope::Spans<T>>>
}

impl<T:Clone> Spans<T> {
    pub fn len(&self) -> Bytes {
        Bytes(self.rc.borrow().len())
    }

    pub fn set(&self, interval:impl IntervalBounds, data:impl Into<T>) {
        let interval    = interval.with_upper_bound(self.len());
        let data        = data.into();
        let mut builder = SpansBuilder::new(interval.size().raw);
        builder.add_span((..),data);
        self.edit(interval,builder.build());
    }

    pub fn set_default(&self, interval:impl IntervalBounds) where T:Default {
        let data : T = default();
        self.set(interval,data);
    }

    pub fn subseq(&self, bounds:impl rope::IntervalBounds) -> rope::tree::Node<rope::SpansInfo<T>> {
        self.rc.borrow().subseq(bounds)
    }

    pub fn focus(&self, bounds:impl rope::IntervalBounds) -> Self {
        let rc = Rc::new(RefCell::new(self.subseq(bounds)));
        Self {rc}
    }

    pub fn to_vector(&self) -> Vec<(rope::Interval,T)> {
        self.rc.borrow().iter().map(|t| (t.0,t.1.clone())).collect_vec()
    }

    pub fn edit
    (&self, bounds:impl rope::IntervalBounds, new:impl Into<rope::tree::Node<rope::SpansInfo<T>>>) {
        self.rc.borrow_mut().edit(bounds,new)
    }

    pub fn raw(&self) -> rope::Spans<T> {
        self.rc.borrow().clone()
    }
}
