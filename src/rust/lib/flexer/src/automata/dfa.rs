use crate::automata::dict::Dict;
use crate::automata::state;

use std::collections::HashMap;

#[derive(Clone,Debug)]
pub struct DFA {
  pub vocabulary             : Dict,
  pub links                  : Vec<Vec<usize>>,
  pub end_state_priority_map : HashMap<usize,state::Desc>
}
