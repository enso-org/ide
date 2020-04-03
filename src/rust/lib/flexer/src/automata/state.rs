use std::ops::Range;

pub(crate) const MISSING: i64 = -1;

#[derive(Clone,Debug,Default)]
pub struct State {
  pub links : LinkRegistry,
  pub rule  : Option<String>,
}

#[derive(Clone,Debug)]
pub struct Desc {
    pub priority : usize,
    pub rule     : String,
}

#[derive(Clone,Debug,Default)]
pub struct LinkRegistry {
    pub epsilon : Vec<usize>,
    pub ranged  : TreeRangeMap<usize, usize>, // TODO use different structure
}

impl LinkRegistry {
    pub(crate) fn add(&mut self, target:usize) {
        self.epsilon += target
    }
    pub(crate) fn add_range(&mut self, target:usize, range:Range<i64>) {
        if range.start <= range.end {
          self.ranged.put(range, target)
        }
    }
}
