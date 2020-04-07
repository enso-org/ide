use std::ops::Range;
use std::collections::HashMap;

pub const MISSING:usize = usize::max_value();

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
    pub targets : HashMap<Range<i64>,usize>,
}

#[derive(Clone,Debug,Default)]
pub struct TreeRangeMapStub();
