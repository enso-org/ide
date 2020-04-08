use std::ops::Range;
use std::collections::BTreeSet;
use itertools::Itertools;

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct Dict {
    pub divisions: BTreeSet<i64>
}

impl Dict {
    pub fn new(iter:&[i64]) -> Self {
        Dict {divisions:iter.iter().cloned().collect()}
    }

    pub fn insert(&mut self, range:Range<i64>) {
        self.divisions.insert(range.start);
        self.divisions.insert(range.end + 1);
    }

    pub fn len(&self) -> usize {
        self.divisions.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() <= 0
    }

    pub fn ranges(&self) -> impl Iterator<Item=Range<i64>> + '_ {
        self.divisions.iter().tuple_windows().map(|(&s,&e)| s..e )
    }
}
