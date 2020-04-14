use crate::automata::alphabet::Alphabet;
use crate::automata::state::StateId;



/// Description of an end state.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct EndState {
    /// Priority of enstate.
    pub priority : usize,
    /// Name of the state. See also State::name
    pub name     : String,
}

/// DFA automata with a set of symbols, states and transitions.
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct DFA {
    /// Set of all valid input symbols.
    pub alphabet   : Alphabet,
    /// A (state X symbol => state) transition matrix.
    pub links      : Vec<Vec<StateId>>,
    /// An (state => Option<EndState>) EndState map.
    pub end_states : Vec<Option<EndState>>,
}
