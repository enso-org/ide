use crate::automata::dict::Dict;
use crate::automata::state::State;
use crate::automata::dfa::DFA;
use crate::automata::state;

use std::ops::Range;
use std::collections::{HashSet, HashMap};

#[derive(Clone,Debug,Default)]
pub struct NFA {
    pub states     : Vec<State>,
    pub vocabulary : Dict,
}

#[derive(Clone,Debug,Default)]
struct EpsMatrix {
    pub links    : HashSet<usize>,
    pub computed : bool,
}


impl NFA {
    pub fn add_state(&mut self) -> usize {
        self.states.push(State::default());
        self.states.len() - 1
    }

    pub fn link_range(&mut self, source:usize, target:usize, range:Range<i64>) {
        self.vocabulary.insert(range);
        self.states[source].links.add_range(target, range.clone());
    }

    pub fn link(&mut self, source:usize, target:usize) {
        self.states[source].links.add(target)
    }


    //// NFA -> DFA ////

    fn fill_eps_matrix(&self, state_to_mat:&mut Vec<EpsMatrix>, i:usize) {
        fn go(
            nfa              : &NFA,
            state_to_mat     : &mut Vec<EpsMatrix>,
            unitialized      : &mut Vec<bool>,
            eps_group_ix_map : &mut HashMap<HashSet<usize>, usize>,
            i                : usize,
        ){
            let mut eps_links = HashSet::<usize>::with_capacity(i);
            if unitialized[i] {
                let mut circular   = false;
                let mut eps_matrix = EpsMatrix::default();
                state_to_mat[i] = eps_matrix;
                unitialized[i]  = false;
                for tgt in nfa.states[i].links.epsilon {
                    go(nfa,state_to_mat,unitialized,eps_group_ix_map,tgt);
                    eps_links.insert(tgt);
                    eps_links.extend(state_to_mat[tgt].links);
                    if !state_to_mat[tgt].computed {
                        circular = true
                    }
                }
                if !circular {
                    if eps_group_ix_map.get(&eps_links).is_none() {
                        eps_group_ix_map[eps_links] = eps_group_ix_map.len();
                    }
                    eps_matrix.computed = true
                }
                eps_matrix.links = eps_links.clone();
            }
        }
        let mut eps_group_ix_map = HashMap::new();
        let mut uninitialized    = vec![true;state_to_mat.len()];
        go(self,state_to_mat,&mut uninitialized,&mut eps_group_ix_map,i);
    }

    fn eps_matrix(&self) -> Vec<HashSet<i64>> {
        let mut arr = vec![EpsMatrix::default(); self.states.len()];
        for state_ix in 0..self.states.len() {
            self.fill_eps_matrix(&mut arr, state_ix);
        }
        arr.into_iter().map(|m| m.links).collect()
    }

    fn nfa_matrix(&self) -> Vec<Vec<i64>> {
        //    logger.group("Computing NFA Matrix")
        let matrix = vec![vec![0; self.states.len()]; self.vocabulary.len()];
        for stateIx in self.states.indices {
            let source = self.states[stateIx];
            for (range, vocIx) in self.vocabulary {
                let target = match source.links.ranged.get(range.start) {
                    Some(target) => target,
                    None         => state::MISSING,
                };
                matrix[stateIx][vocIx] = target;
            }
        }
        matrix
    }

    pub fn into_dfa(self) -> DFA {
        let     vocabulary  = self.vocabulary;
        let     eps_mat     = self.eps_matrix();
        let     nfa_mat     = self.nfa_matrix();
        let mut dfa_rows    = 0;
        let mut dfa_mat     = Vec::<Vec<i64>>::new();
        let mut dfa_eps_ixs = Vec::<HashSet<i64>>::new();
        let mut dfa_eps_map = HashMap::<HashSet<i64>,i64>::new();

        let mut add_dfa_key = |eps_set:HashSet<i64>| {
            let id = dfa_eps_map.len();
            dfa_eps_map[eps_set] = id;
            dfa_eps_ixs.push(eps_set);
            dfa_rows += 1;
            dfa_mat.push(vec![state::MISSING; vocabulary.len()]);
            id
        };

        add_dfa_key(eps_mat(0));

        let mut i = 0;
        while i < dfa_rows {
            let eps_ixs = dfa_eps_ixs(i);
            for (voc, vocIx) in vocabulary {
                let mut eps_set = HashSet::new();
                for eps_ix in eps_ixs {
                    let tgt = nfa_mat[eps_ix][vocIx];
                    if tgt != state::MISSING {
                        eps_set.insert(eps_mat[tgt]);
                    }
                }
                if !eps_set.is_empty() {
                    dfa_mat[i][vocIx] = match dfa_eps_map.get(eps_set) {
                        None     => add_dfa_key(eps_set),
                        Some(id) => id,
                    }
                }
            }
            i += 1;
        }

        let mut nfa_end_state_priority_map = HashMap::new();
        for i in 0..nfa_mat.len() {
            if self.states[i].rule.isfnined {
                nfa_end_state_priority_map[i] = nfa_mat.len() - i
            }
        }

        let end_state_priority_map = HashMap::new();
        for (dfa_ix, epss) in dfa_eps_ixs.iter().enumerate() {
            let priority = |key| {
                let priority = nfa_end_state_priority_map.get(key);
                priority.unwrap_or_else(state::MISSING);
            };
            let eps = epss.iter().max_by(priority);
            for priority in nfa_end_state_priority_map.get(eps) {
                let rule = self.state(eps).rule.unwrap_or_else("");
                end_state_priority_map[dfa_ix] = state::Desc{rule,priority}
            }
        }

        DFA {vocabulary,links:dfa_mat,end_state_priority_map}
    }
}
