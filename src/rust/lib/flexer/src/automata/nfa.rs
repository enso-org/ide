use crate::automata::alphabet::Alphabet;
use crate::automata::dfa::DFA;
use crate::automata::dfa::Callback;
use crate::automata::state::Link;
use crate::automata::state::Symbol;
use crate::automata::state::State;
use crate::automata::state;

use std::collections::HashMap;
use std::collections::BTreeSet;
use std::ops::RangeInclusive;



// ========================================
// === Nondeterministic Finite Automata ===
// ========================================

/// Type alias for a state Id based on set of states.
/// It is used during NFA -> DFA transformation where
/// multiple states can merge together, thanks to epsilon links.
type StateSetId = BTreeSet<state::Id>;

/// NFA automata with a set of symbols, states and transitions.
/// Nondeterministic Finite Automata is a finite-state machine
/// that accepts or rejects a given sequence of symbols.
/// Compared to `DFA`, NFA can transition into multiple new states
/// without reading any symbol (so called epsilon link / transition),
///   ___              ___         ___              ___              ___
///  | 0 | -- 'N' --> | 1 | ----> | 2 | -- 'F' --> | 3 | -- 'A' --> | 4 |
///   ‾‾‾              ‾‾‾         ‾‾‾              ‾‾‾              ‾‾‾
/// More information at: https://en.wikipedia.org/wiki/Deterministic_finite_automaton
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct NFA {
    /// Finite set of all valid input symbols.
    pub alphabet: Alphabet,
    /// Set of named NFA states with (epsilon) transitions.
    pub states: Vec<State>,
}

impl NFA {
    /// Adds a new state to NFA and returns it's Id.
    pub fn new_state(&mut self) -> state::Id {
        let id = self.states.len();
        self.states.push(State::default());
        state::Id {id}
    }

    /// Creates an epsilon transition between two states.
    /// Whenever the automata happens to be in `source` state
    /// it can immediatelly (but does not have to)  move to `target` state.
    pub fn connect(&mut self, source:state::Id, target:state::Id) {
        self.states[source.id].epsilon_links.push(target);
    }

    /// Creates an ordinary transition (for a range of symbols) between two states.
    /// If any symbol from such range happens to be on input when the automata
    /// is in `source` state, it will immediatelly move to `target` state.
    pub fn connect_by
    (&mut self, source:state::Id, target:state::Id, symbols:&RangeInclusive<Symbol>) {
        self.alphabet.insert(symbols.clone());
        self.states[source.id].links.push(Link{symbols:symbols.clone(), target});
    }


    // === NFA -> DFA ===

    /// Merges states that are connected by epsilon links.
    /// The algorithm is based on: https://www.youtube.com/watch?v=taClnxU-nao
    fn eps_matrix(&self) -> Vec<StateSetId> {
        fn fill_eps_matrix
        ( nfa      : &NFA
        , states   : &mut Vec<StateSetId>
        , computed : &mut Vec<bool>
        , visited  : &mut Vec<bool>
        , state    : state::Id
        ) {
            let mut state_set = StateSetId::new();
            let mut circular  = false;
            visited[state.id] = true;
            state_set.insert(state);
            for &target in &nfa.states[state.id].epsilon_links {
                if !visited[target.id] {
                    fill_eps_matrix(nfa,states,computed,visited,target);
                }
                state_set.insert(target);
                state_set.extend(states[target.id].iter());
                if !computed[target.id] {
                    circular = true
                }
            }
            if !circular {
                computed[state.id] = true
            }
            states[state.id] = state_set;
        }

        let mut states   = vec![StateSetId::new(); self.states.len()];
        let mut computed = vec![false; self.states.len()];
        for id in 0..self.states.len() {
            let mut visited = vec![false; states.len()];
            fill_eps_matrix(self,&mut states,&mut computed,&mut visited,state::Id{id});
        }
        states
    }

    /// Computes a transition matrix (state X symbol => state) for NFA.
    /// Ignores epsilon links.
    fn nfa_matrix(&self) -> Vec<Vec<state::Id>> {
        let symbols_len = self.alphabet.symbols.len();
        let states_len  = self.states.len();
        let mut matrix  = vec![vec![state::Id {id:0}; symbols_len]; states_len];

        for (state_ix, source) in self.states.iter().enumerate() {
            let targets = source.targets(&self.alphabet);
            for (voc_ix, &target) in targets.iter().enumerate() {
                matrix[state_ix][voc_ix] = target;
            }
        }
        matrix
    }
}

impl From<&NFA> for DFA {
    /// Transforms NFA into DFA.
    /// The algorithm is based on: https://www.youtube.com/watch?v=taClnxU-nao
    fn from(nfa:&NFA) -> Self {
        let     nfa_mat     = nfa.nfa_matrix();
        let     eps_mat     = nfa.eps_matrix();
        let mut dfa_mat     = Vec::<Vec<state::Id>>::new();
        let mut dfa_eps_ixs = Vec::<StateSetId>::new();
        let mut dfa_eps_map = HashMap::<StateSetId, state::Id>::new();

        dfa_eps_ixs.push(eps_mat[0].clone());
        dfa_eps_map.insert(eps_mat[0].clone(), state::Id {id:0});

        let mut i = 0;
        while i < dfa_eps_ixs.len()  {
            dfa_mat.push(vec![state::INVALID; nfa.alphabet.symbols.len()]);
            for voc_ix in 0..nfa.alphabet.symbols.len() {
                let mut eps_set = StateSetId::new();
                for &eps_ix in &dfa_eps_ixs[i] {
                    let tgt = nfa_mat[eps_ix.id][voc_ix];
                    if tgt != state::INVALID {
                        eps_set.extend(eps_mat[tgt.id].iter());
                    }
                }
                if !eps_set.is_empty() {
                    dfa_mat[i][voc_ix] = match dfa_eps_map.get(&eps_set) {
                        Some(&id) => id,
                        None => {
                            let id = state::Id {id:dfa_eps_ixs.len()};
                            dfa_eps_ixs.push(eps_set.clone());
                            dfa_eps_map.insert(eps_set,id);
                            id
                        },
                    };
                }
            }
            i += 1;
        }

        let mut callbacks = vec![None; dfa_eps_ixs.len()];
        let     priority  = dfa_eps_ixs.len();
        for (dfa_ix, epss) in dfa_eps_ixs.iter().enumerate() {
            let has_name = |&&key:&&state::Id| nfa.states[key.id].name.is_some();
            if let Some(&eps) = epss.iter().find(has_name) {
                let rule  = nfa.states[eps.id].name.as_ref().cloned().unwrap();
                callbacks[dfa_ix] = Some(Callback {name:rule,priority});
            }
        }

        DFA {alphabet:nfa.alphabet.clone(),links:dfa_mat,callbacks}
    }
}
