use crate::automata::dict::Dict;
use crate::automata::state::State;
use crate::automata::dfa::DFA;
use crate::automata::{state, dfa};

use std::ops::Range;
use std::collections::{HashMap, BTreeSet};

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct NFA {
    pub states     : Vec<State>,
    pub vocabulary : Dict,
}

#[derive(Clone,Debug,Default)]
struct EpsMatrix {
    pub links       : BTreeSet<usize>,
    pub is_computed : bool,
}

impl NFA {
    pub fn add_state(&mut self) -> usize {
        self.states.push(State::default());
        self.states.len() - 1
    }

    pub fn set_link_range(&mut self, source:usize, target:usize, range:&Range<i64>) {
        self.vocabulary.insert(range.clone());
        self.states[source].link_target.insert(range.clone(), target);
    }

    pub fn set_link(&mut self, source:usize, target:usize) {
        self.states[source].link_epsilon.push(target);
    }


    //// NFA -> DFA ////

    fn fill_eps_matrix(&self, state_to_mat:&mut Vec<EpsMatrix>, i:usize) {
        fn go(
            nfa              : &NFA,
            state_to_mat     : &mut Vec<EpsMatrix>,
            unitialized      : &mut Vec<bool>,
            eps_group_ix_map : &mut HashMap<BTreeSet<usize>, usize>,
            i                : usize,
        ){
            let mut eps_links = BTreeSet::<usize>::new();
            if unitialized[i] {
                let mut circular = false;
                unitialized[i]   = false;
                state_to_mat[i]  = EpsMatrix::default();
                for &tgt in &nfa.states[i].link_epsilon {
                    go(nfa,state_to_mat,unitialized,eps_group_ix_map,tgt);
                    eps_links.insert(tgt);
                    eps_links.extend(state_to_mat[tgt].links.iter());
                    if !state_to_mat[tgt].is_computed {
                        circular = true
                    }
                }
                if !circular {
                    if !eps_group_ix_map.contains_key(&eps_links) {
                        eps_group_ix_map.insert(eps_links.clone(), eps_group_ix_map.len());
                    }
                    state_to_mat[i].is_computed = true
                }
                state_to_mat[i].links = eps_links;
            }
        }
        let mut eps_group_ix_map = HashMap::new();
        let mut uninitialized    = vec![true;state_to_mat.len()];
        go(self,state_to_mat,&mut uninitialized,&mut eps_group_ix_map,i);
    }

    fn eps_matrix(&self) -> Vec<BTreeSet<usize>> {
        let mut arr = vec![EpsMatrix::default(); self.states.len()];
        for state_ix in 0..self.states.len() {
            self.fill_eps_matrix(&mut arr, state_ix);
        }
        arr.into_iter().map(|m| m.links).collect()
    }

    fn nfa_matrix(&self) -> Vec<Vec<usize>> {
        let mut matrix = vec![vec![0; self.states.len()]; self.vocabulary.len()];
        for (state_ix, source) in self.states.iter().enumerate() {
            for (voc_ix, range) in self.vocabulary.ranges().enumerate() {
                let target = match source.link_target.get(&range) {
                    Some(&target) => target,
                    None          => state::MISSING,
                };
                matrix[state_ix][voc_ix] = target;
            }
        }
        matrix
    }
    fn add_dfa_key(
        eps_set     : BTreeSet<usize>,
        vocabulary  : &Dict,
        dfa_eps_map : &mut HashMap<BTreeSet<usize>,usize>,
        dfa_eps_ixs : &mut Vec<BTreeSet<usize>>,
        dfa_mat     : &mut Vec<Vec<usize>>,
        dfa_rows    : &mut usize,
    ) -> usize
    {
        let id = dfa_eps_map.len();
        dfa_eps_map.insert(eps_set.clone(), id);
        dfa_eps_ixs.push(eps_set);
        *dfa_rows += 1;
        dfa_mat.push(vec![state::MISSING; vocabulary.len()]);
        id
    }

    pub fn to_dfa(&self) -> DFA {
        let     eps_mat     = self.eps_matrix();
        let     nfa_mat     = self.nfa_matrix();
        let mut dfa_rows    = 0;
        let mut dfa_mat     = Vec::<Vec<usize>>::new();
        let mut dfa_eps_ixs = Vec::<BTreeSet<usize>>::new();
        let mut dfa_eps_map = HashMap::<BTreeSet<usize>,usize>::new();

        Self::add_dfa_key(eps_mat[0].clone(),&self.vocabulary,&mut dfa_eps_map,&mut dfa_eps_ixs,&mut dfa_mat,&mut dfa_rows);

        let mut i = 0;
        while i < dfa_rows {
            for (voc_ix, _) in self.vocabulary.ranges().enumerate() {
                let mut eps_set = BTreeSet::<usize>::new();
                for &eps_ix in &dfa_eps_ixs[i] {
                    let tgt = nfa_mat[eps_ix][voc_ix];
                    if tgt != state::MISSING {
                        eps_set.extend(eps_mat[tgt].iter());
                    }
                }
                if !eps_set.is_empty() {
                    dfa_mat[i][voc_ix] = match dfa_eps_map.get(&eps_set) {
                        None      => Self::add_dfa_key(eps_set,&self.vocabulary,&mut dfa_eps_map,&mut dfa_eps_ixs,&mut dfa_mat,&mut dfa_rows),
                        Some(&id) => id,
                    };
                }
            }
            i += 1;
        }

        let mut nfa_end_state_priority_map = HashMap::<usize,usize>::new();
        for i in 0..nfa_mat.len() {
            if self.states[i].rule.is_some() {
                nfa_end_state_priority_map.insert(i, nfa_mat.len() - i);
            }
        }

        let mut priorities = dfa::EndStatePriorityMap::default();
        for (dfa_ix, epss) in dfa_eps_ixs.iter().enumerate() {
            let priority = |key:&&usize| {
                let priority = nfa_end_state_priority_map.get(*key);
                priority.cloned().unwrap_or_default()
            };
            if let Some(&eps) = epss.iter().max_by_key(priority) {
                if let Some(&priority) = nfa_end_state_priority_map.get(&eps) {
                    let rule  = self.states[eps].rule.as_ref().cloned().unwrap_or_default();
                    let state = state::Desc{rule,priority};
                    priorities.val.insert(dfa_ix, state);
                }
            }
        }

        DFA {vocabulary:self.vocabulary.clone(),links:dfa_mat,priorities}
    }
}

// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    // use super::*;
}
