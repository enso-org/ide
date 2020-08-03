//! Text selection and carets implementation.

use crate::prelude::*;

use crate::buffer::data::unit::*;
use crate::buffer::data::Range;



// =====================
// === SelectionData ===
// =====================


#[derive(Clone,Copy,PartialEq,Eq,Debug,Default)]
#[allow(missing_docs)]
pub struct SlectionData<T=Location> {
    pub start : T,
    pub end   : T
}


// =================
// === Selection ===
// =================


/// Text selection. In case the `start` and `end` offsets are equal, the selection is interpreted as
/// a caret. The `column` field is a saved horizontal position used primarily for line up/down
/// movement. Please note that the start of the selection is not always smaller then its end.
/// If the selection was dragged from right to left, the start byte offset will be bigger than the
/// end. Use the `min` and `max` methods to discover the edges.
#[derive(Clone,Copy,PartialEq,Eq,Debug,Default)]
#[allow(missing_docs)]
pub struct Selection<T=Location> {
    pub start : T,
    pub end   : T,
    pub id    : usize,
}

impl<T:Copy+Ord+Eq> Selection<T> {
    /// Constructor.
    pub fn new(start:T, end:T, id:usize) -> Self {
        Self {start,end,id}
    }

    /// Cursor constructor (zero-length selection).
    pub fn new_cursor(offset:T, id:usize) -> Self {
        Self::new(offset,offset,id)
    }

    /// Range of this selection.
    pub fn range(&self) -> Range<T> {
        (self.min() .. self.max()).into()
    }

//    /// Size of this selection in bytes.
//    pub fn size(&self) -> Bytes {
//        self.end - self.start
//    }

    /// Gets the earliest offset within the selection, ie the minimum of both edges.
    pub fn min(self) -> T {
        std::cmp::min(self.start,self.end)
    }

    /// Gets the latest offset within the selection, ie the maximum of both edges.
    pub fn max(self) -> T {
        std::cmp::max(self.start,self.end)
    }

    pub fn with_start(&self, start:T) -> Self {
        Self {start,..*self}
    }

    pub fn with_end(&self, end:T) -> Self {
        Self {end,..*self}
    }

    pub fn map_start(&self, f:impl Fn(T)->T) -> Self {
        self.with_start(f(self.start))
    }

    pub fn map_end(&self, f:impl Fn(T)->T) -> Self {
        self.with_end(f(self.end))
    }

    pub fn map(&self, f:impl Fn(T)->T) -> Self {
        self.with_start(f(self.start)).with_end(f(self.end))
    }

    pub fn as_caret(&self) -> Self {
        let end = self.start;
        Self {end,..*self}
    }

    /// Determines whether the selection is a caret (ie has an empty interior).
    pub fn is_caret(self) -> bool {
        self.start == self.end
    }

//    /// Returns a selection with the given horizontal position.
//    pub fn with_column(self, column:Option<Column>) -> Self {
//        Self {column,..self}
//    }

    /// Indicate whether this region should merge with the next.
    /// Assumption: regions are sorted (self.min() <= other.min())
    pub fn should_merge_sorted(self, other:Selection<T>) -> bool {
        let non_zero_overlap = other.min() < self.max();
        let zero_overlap     = (self.is_caret() || other.is_caret()) && other.min() == self.max();
        non_zero_overlap || zero_overlap
    }

    /// Merge self with an overlapping region. Retains direction of self.
    pub fn merge_with(self, other:Selection<T>) -> Selection<T> {
        let is_forward  = self.end >= self.start;
        let new_min     = std::cmp::min(self.min(), other.min());
        let new_max     = std::cmp::max(self.max(), other.max());
        let (start,end) = if is_forward { (new_min,new_max) } else { (new_max,new_min) };
        Selection::new(start,end,self.id)
    }
}



// =============
// === Group ===
// =============

/// A set of zero or more selection regions, representing a selection state.
#[derive(Clone,Debug,Default)]
pub struct Group {
    sorted_regions: Vec<Selection>,
}

impl Deref for Group {
    type Target = [Selection];
    fn deref(&self) -> &[Selection] {
        &self.sorted_regions
    }
}

impl DerefMut for Group {
    fn deref_mut(&mut self) -> &mut [Selection] {
        &mut self.sorted_regions
    }
}

impl Group {
    /// Constructor.
    pub fn new() -> Group {
        Group::default()
    }

    pub fn to_carets(&self) -> Group {
        Self {sorted_regions : self.sorted_regions.iter().map(|t| t.as_caret()).collect()}
    }

    pub fn newest(&self) -> Option<&Selection> {
        self.sorted_regions.iter().max_by(|x,y| x.id.cmp(&y.id))
    }

    pub fn oldest(&self) -> Option<&Selection> {
        self.sorted_regions.iter().min_by(|x,y| x.id.cmp(&y.id))
    }

    pub fn newest_mut(&mut self) -> Option<&mut Selection> {
        self.sorted_regions.iter_mut().max_by(|x,y| x.id.cmp(&y.id))
    }

    pub fn oldest_mut(&mut self) -> Option<&mut Selection> {
        self.sorted_regions.iter_mut().min_by(|x,y| x.id.cmp(&y.id))
    }

    /// Add a region to the selection. This method implements merging logic.
    ///
    /// Two non-caret regions merge if their interiors intersect. Merely touching at the edges does
    /// not cause a merge. A caret merges with a non-caret if it is in the interior or on either
    /// edge. Two carets merge if they are the same offset.
    ///
    /// Performance note: should be O(1) if the new region strictly comes after all the others in
    /// the selection, otherwise O(n).
    pub fn add(&mut self, region:Selection) {
        let mut ix = self.selection_index_on_the_left_to(region.min());
        if ix == self.sorted_regions.len() {
            self.sorted_regions.push(region);
            return;
        }
        let mut region = region;
        let mut end_ix = ix;
        if self.sorted_regions[ix].min() <= region.min() {
            if self.sorted_regions[ix].should_merge_sorted(region) {
                region = region.merge_with(self.sorted_regions[ix]);
            } else {
                ix += 1;
            }
            end_ix += 1;
        }

        let max_ix = self.sorted_regions.len();
        while end_ix < max_ix && region.should_merge_sorted(self.sorted_regions[end_ix]) {
            region = region.merge_with(self.sorted_regions[end_ix]);
            end_ix += 1;
        }

        if ix == end_ix {
            self.sorted_regions.insert(ix,region);
        } else {
            let start = ix + 1;
            let len   = end_ix - ix - 1;
            self.sorted_regions[ix] = region;
            self.sorted_regions.drain(start..start+len);
        }
    }

    /// The smallest index so that offset > region.max() for all preceding
    /// regions.
    pub fn selection_index_on_the_left_to(&self, location:Location) -> usize {
        if self.sorted_regions.is_empty() || location > self.sorted_regions.last().unwrap().max() {
            self.sorted_regions.len()
        } else {
            self.sorted_regions.binary_search_by(|r| r.max().cmp(&location)).unwrap_both()
        }
    }
}

impl From<Selection> for Group {
    fn from(t:Selection) -> Self {
        let sorted_regions = vec![t];
        Self {sorted_regions}
    }
}

impl From<Option<Selection>> for Group {
    fn from(t:Option<Selection>) -> Self {
        t.map(|s|s.into()).unwrap_or_default()
    }
}

impl<'t> IntoIterator for &'t Group {
    type Item     = &'t Selection;
    type IntoIter = slice::Iter<'t,Selection>;
    fn into_iter(self) -> Self::IntoIter {
        self.sorted_regions.iter()
    }
}

impl FromIterator<Selection> for Group {
    fn from_iter<T:IntoIterator<Item=Selection>>(iter:T) -> Self {
        let mut group = Group::new();
        for selection in iter {
            group.add(selection);
        }
        group
    }
}
