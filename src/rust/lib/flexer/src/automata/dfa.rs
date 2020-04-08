use crate::automata::dict::Dict;
use crate::automata::state;

use std::collections::HashMap;



#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct DFA {
    pub vocabulary : Dict,
    pub links      : Vec<Vec<usize>>,
    pub priorities : EndStatePriorityMap,
}

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct EndStatePriorityMap {
    pub val: HashMap<usize,state::Desc>
}

impl EndStatePriorityMap {
    pub fn new(iter:&[(usize,state::Desc)]) -> Self{
        EndStatePriorityMap {val:iter.iter().cloned().collect()}
    }
}
