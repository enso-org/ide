use crate::automata::alphabet::Alphabet;
use crate::automata::dfa::DFA;
use crate::automata::dfa::EndState;
use crate::automata::state::Link;
use crate::automata::state::Symbol;
use crate::automata::state::StateId;
use crate::automata::state::State;
use crate::automata::state;

use std::collections::HashMap;
use std::collections::BTreeSet;
use std::ops::Range;



type StateSetId = BTreeSet<StateId>;

/// NFA automata with a set of symbols, states and transitions.
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct NFA {
    /// Set of NFA states.
    pub states   : Vec<State>,
    /// Set of valid input symbols.
    pub alphabet : Alphabet,
}

impl NFA {
    /// Adds new state to NFA and returns it's Id.
    pub fn new_state(&mut self) -> StateId {
        self.states.push(State::default());
        self.states.len() - 1
    }

    /// Creates an epsilon transition between two states.
    pub fn connect(&mut self, source:StateId, target:StateId) {
        self.states[source].epsilon_links.push(target);
    }

    /// Creates a transition (for a range of symbols) between two states.
    pub fn connect_by(&mut self, source:StateId, target:StateId, symbols:&Range<Symbol>) {
        self.alphabet.insert(symbols.clone());
        self.states[source].links.push(Link{symbols:symbols.clone(), target});
    }


    //// NFA -> DFA ////

    fn fill_eps_matrix(&self, states:&mut Vec<StateSetId>, computed:&mut Vec<bool>, state:StateId) {
        fn go(
            nfa      : &NFA,
            states   : &mut Vec<StateSetId>,
            computed : &mut Vec<bool>,
            visisted : &mut Vec<bool>,
            state    : StateId,
        ){
            let mut stateset = StateSetId::new();
            let mut circular = false;
            visisted[state]  = true;
            stateset.insert(state);
            for &tgt in &nfa.states[state].epsilon_links {
                if !visisted[tgt] {
                    go(nfa, states, computed, visisted, tgt);
                }
                stateset.insert(tgt);
                stateset.extend(states[tgt].iter());
                if !computed[tgt] {
                    circular = true
                }
            }
            if !circular {
                computed[state] = true
            }
            states[state] = stateset;

        }
        let mut visited = vec![false; states.len()];
        go(self, states, &mut visited, computed, state);
    }

    fn eps_matrix(&self) -> Vec<StateSetId> {
        let mut states   = vec![StateSetId::new(); self.states.len()];
        let mut computed = vec![false; self.states.len()];
        for state in 0..self.states.len() {
            self.fill_eps_matrix(&mut states,&mut computed, state);
        }
        states
    }

    fn nfa_matrix(&self) -> Vec<Vec<StateId>> {
        let mut matrix = vec![vec![0; self.alphabet.symbols.len()]; self.states.len()];
        for (state_ix, source) in self.states.iter().enumerate() {
            println!("{:?} || {:?}", source.links, self.alphabet.symbols.iter().collect::<Vec<_>>());

            for (voc_ix, target) in source.targets(&self.alphabet).into_iter().enumerate() {
                matrix[state_ix][voc_ix] = target;
            }
        }
        matrix
    }

    /// Transforms NFA into DFA.
    /// The algorithm is based on: https://www.youtube.com/watch?v=taClnxU-nao
    pub fn to_dfa(&self) -> DFA {
        let     eps_mat     = self.eps_matrix();
        let     nfa_mat     = self.nfa_matrix();
        let mut dfa_mat     = Vec::<Vec<StateId>>::new();
        let mut dfa_eps_ixs = Vec::<StateSetId>::new();
        let mut dfa_eps_map = HashMap::<StateSetId,StateId>::new();


        println!("{:?}", eps_mat);
        println!("{:?}", nfa_mat);

        dfa_eps_ixs.push(eps_mat[0].clone());
        dfa_eps_map.insert(eps_mat[0].clone(),0);

        let mut i = 0;
        while i < dfa_eps_ixs.len()  {
            dfa_mat.push(vec![state::MISSING; self.alphabet.symbols.len()]);
            for voc_ix in 0..self.alphabet.symbols.len() {
                let mut eps_set = StateSetId::new();
                for &eps_ix in &dfa_eps_ixs[i] {
                    let tgt = nfa_mat[eps_ix][voc_ix];
                    if tgt != state::MISSING {
                        eps_set.extend(eps_mat[tgt].iter());
                    }
                }
                println!("{:?}", eps_set);
                if !eps_set.is_empty() {
                    dfa_mat[i][voc_ix] = match dfa_eps_map.get(&eps_set) {
                        Some(&id) => id,
                        None => {
                            let id = dfa_eps_ixs.len();
                            dfa_eps_ixs.push(eps_set.clone());
                            dfa_eps_map.insert(eps_set, id);
                            id
                        },
                    };
                }
            }
            i += 1;
        }

        let mut end_states = vec![None;dfa_eps_ixs.len()];
        let     priority   = dfa_eps_ixs.len();
        for (dfa_ix, epss) in dfa_eps_ixs.iter().enumerate() {
            let has_name = |&&key:&&StateId| self.states[key].name.is_some();
            if let Some(&eps) = epss.iter().find(has_name) {
                let rule  = self.states[eps].name.as_ref().cloned().unwrap();
                end_states[dfa_ix] = Some(EndState{name:rule,priority});
            }
        }

        DFA {alphabet:self.alphabet.clone(),links:dfa_mat,end_states}
    }
}