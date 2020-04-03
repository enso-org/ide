use crate::automata::dict::Dict;
use crate::automata::state;

use std::collections::HashMap;

#[derive(Clone,Debug)]
pub struct DFA {
  vocabulary             : Dict,
  links                  : Vec<Vec<i64>>,
  end_state_priority_map : HashMap<i64,state::Desc>
}
