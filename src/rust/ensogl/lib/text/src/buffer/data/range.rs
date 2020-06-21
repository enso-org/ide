use crate::prelude::*;

use super::rope;
use super::unit::*;



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



// ===================
// === RangeBounds ===
// ===================

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
