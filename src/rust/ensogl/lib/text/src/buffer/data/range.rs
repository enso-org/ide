use crate::prelude::*;

use super::rope;
use super::unit::*;



// =============
// === Range ===
// =============

/// A (half-open) range bounded inclusively below and exclusively above [start,end).
///
/// Unlike `std::ops::Range`, this range is strongly typed, implements `Copy`, and contains a lot
/// of utilities for working with bytes ranges for the purpose of text manipulation.
#[derive(Clone,Copy,PartialEq,Eq)]
pub struct Range<T> {
    pub start : T,
    pub end   : T,
}

impl<T> Range<T> {
    /// Constructor.
    pub fn new(start:T, end:T) -> Self {
        Self {start,end}
    }

    /// The size of the range.
    pub fn size(&self) -> T where T : Clone + Sub<T,Output=T> {
        self.end.clone() - self.start.clone()
    }
}

impl Range<Bytes> {
    pub fn into_rope_interval(self) -> rope::Interval {
        self.into()
    }
}

impl<T:Display> fmt::Display for Range<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

impl<T:Debug> fmt::Debug for Range<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}, {:?})", self.start, self.end)
    }
}

impl<T> From<std::ops::Range<T>> for Range<T> {
    fn from(range:std::ops::Range<T>) -> Range<T> {
        let std::ops::Range {start,end} = range;
        Range {start,end}
    }
}


// === Bytes Impls ===

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

impl Range<Bytes> {
    pub fn into_rope_repr(self) -> rope::Interval {
        self.into()
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

/// RangeBounds allows converting all Rust ranges to the `Range` type, including open ranges, like
/// `..`, `a..`, `..b`, and `..=c`. When used for text manipulation, open ranges are clamped between
/// 0 bytes and the total bytes of the text.
pub trait RangeBounds {
    /// Clamp the range to the total bytes of the text/
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
