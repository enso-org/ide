
use crate::model::location;

// =================
// === Selection ===
// =================

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub struct Selection {
    /// The inactive edge of a selection, as a byte offset. When
    /// equal to end, the selection range acts as a caret.
    pub start: usize,

    /// The active edge of a selection, as a byte offset.
    pub end: usize,

    /// A saved horizontal position (used primarily for line up/down movement).
    pub horiz: Option<location::Column>,
}

impl Selection {

    /// Returns a new region.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end, horiz: None }//, affinity: Affinity::default() }
    }

    /// Gets the earliest offset within the region, ie the minimum of both edges.
    pub fn min(self) -> usize {
        std::cmp::min(self.start, self.end)
    }

    /// Gets the latest offset within the region, ie the maximum of both edges.
    pub fn max(self) -> usize {
        std::cmp::max(self.start, self.end)
    }

    /// Determines whether the region is a caret (ie has an empty interior).
    pub fn is_caret(self) -> bool {
        self.start == self.end
    }

    /// Returns a region with the given horizontal position.
    pub fn with_horiz(self, horiz: Option<location::Column>) -> Self {
        Self { horiz, ..self }
    }

    // Indicate whether this region should merge with the next.
    // Assumption: regions are sorted (self.min() <= other.min())
    pub fn should_merge(self, other: Selection) -> bool {
        other.min() < self.max()
            || ((self.is_caret() || other.is_caret()) && other.min() == self.max())
    }

    // Merge self with an overlapping region.
    // Retains direction of self.
    pub fn merge_with(self, other: Selection) -> Selection {
        let is_forward = self.end >= self.start;
        let new_min = std::cmp::min(self.min(), other.min());
        let new_max = std::cmp::max(self.max(), other.max());
        let (start, end) = if is_forward { (new_min, new_max) } else { (new_max, new_min) };
        // Could try to preserve horiz/affinity from one of the
        // sources, but very likely not worth it.
        Selection::new(start, end)
    }

}