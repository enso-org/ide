use std::ops::Range;
use std::collections::HashMap;


pub const MISSING:usize = usize::max_value();

#[derive(Clone,Debug)]
pub struct Id {
    pub val: usize
}

#[derive(Clone,Debug)]
pub struct Rule {
    pub val: String
}

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct State {
  pub link_epsilon : Vec<usize>,
  pub link_target  : HashMap<Range<i64>,usize>,
  pub rule         : Option<String>,
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Desc {
    pub priority : usize,
    pub rule     : String,
}

// impl<T:Into<String>> From<T> for Rule {
//     fn from(t: T) -> Self {
//         Rule{val:t.into()}
//     }
// }

impl State {
    pub fn link_epsilon(iter:&[usize]) -> Self {
        State { link_epsilon: iter.iter().cloned().collect(), ..Default::default() }
    }

    pub fn link_target(iter:&[(Range<i64>,usize)]) -> Self {
        State { link_target: iter.iter().cloned().collect(), ..Default::default() }
    }

    pub fn named(mut self, name:&str) -> Self {
        self.rule = Some(name.to_owned());
        self
    }
}