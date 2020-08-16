//! The structure for defining deterministic finite automata.

use crate::prelude::*;

use crate::symbol::Symbol;
use crate::alphabet;
use crate::state::State;
use crate::state;
use crate::data::matrix::Matrix;
use crate::nfa;
use crate::nfa::Nfa;



// =====================================
// === Deterministic Finite Automata ===
// =====================================

/// The definition of a [DFA](https://en.wikipedia.org/wiki/Deterministic_finite_automaton) for a
/// given set of symbols, states, and transitions.
///
/// A DFA is a finite state automaton that accepts or rejects a given sequence of symbols by
/// executing on a sequence of states _uniquely_ determined by the sequence of input symbols.
///
/// ```text
///  ┌───┐  'D'  ┌───┐  'F'  ┌───┐  'A'  ┌───┐
///  │ 0 │ ----> │ 1 │ ----> │ 2 │ ----> │ 3 │
///  └───┘       └───┘       └───┘       └───┘
/// ```
#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct Dfa {
    /// A set of disjoint intervals over the allowable input alphabet.
    pub alphabet : alphabet::SealedSegmentation,
    /// The transition matrix for the Dfa.
    ///
    /// It represents a function of type `(state, symbol) -> state`, returning the identifier for
    /// the new state.
    ///
    /// For example, the transition matrix for an automaton that accepts the language
    /// `{"A" | "B"}*"` would appear as follows, with `-` denoting
    /// [the invalid state](state::INVALID). The leftmost column encodes the input state, while the
    /// topmost row encodes the input symbols.
    ///
    /// |   | A | B |
    /// |:-:|:-:|:-:|
    /// | 0 | 1 | - |
    /// | 1 | - | 0 |
    ///
    pub links : Matrix<State>,
    /// A collection of callbacks for each state (indexable in order)
    pub callbacks : Vec<Option<RuleExecutable>>,
}

impl Dfa {
    pub const START_STATE : State = State::new(0);
}

impl Dfa {
    pub fn next_state(&self, current_state:State, symbol:Symbol) -> State {
        self.alphabet.index_of_symbol(symbol).and_then(|ix| {
            self.links.safe_index(ix,current_state.id())
        }).unwrap_or_default()
    }
}


// === Trait Impls ===

impl From<Vec<Vec<usize>>> for Matrix<State> {
    fn from(input:Vec<Vec<usize>>) -> Self {
        let rows       = input.len();
        let columns    = if rows == 0 {0} else {input[0].len()};
        let mut matrix = Self::new(rows,columns);
        for row in 0..rows {
            for column in 0..columns {
                matrix[(row,column)] = State::new(input[row][column]);
            }
        }
        matrix
    }
}



// ================
// === Callback ===
// ================

/// The callback associated with an arbitrary state of a finite automaton.
///
/// It contains the rust code that is intended to be executed after encountering a
/// [`pattern`](super::pattern::Pattern) that causes the associated state transition. This pattern
/// is declared in [`Rule.pattern`](crate::group::rule::Rule::pattern).
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RuleExecutable {
    /// A description of the priority with which the callback is constructed during codegen.
    pub priority: usize,
    /// The rust code that will be executed when running this callback.
    pub code: String,
}



// ===================
// === Conversions ===
// ===================

impl From<&Nfa> for Dfa {
    /// Transforms an Nfa into a Dfa, based on the algorithm described
    /// [here](https://www.youtube.com/watch?v=taClnxU-nao).
    /// The asymptotic complexity is quadratic in number of states.
    fn from(nfa:&Nfa) -> Self {
        let     nfa_mat     = nfa.nfa_matrix();
        let     eps_mat     = nfa.eps_matrix();
        let mut dfa_mat     = Matrix::new(0,nfa.alphabet.divisions.len());
        let mut dfa_eps_ixs = Vec::<nfa::StateSetId>::new();
        let mut dfa_eps_map = HashMap::<nfa::StateSetId,State>::new();

        dfa_eps_ixs.push(eps_mat[0].clone());
        dfa_eps_map.insert(eps_mat[0].clone(),Dfa::START_STATE);

        let mut i = 0;
        while i < dfa_eps_ixs.len()  {
            dfa_mat.new_row();
            for voc_ix in 0..nfa.alphabet.divisions.len() {
                let mut eps_set = nfa::StateSetId::new();
                for &eps_ix in &dfa_eps_ixs[i] {
                    let tgt = nfa_mat[(eps_ix.id(),voc_ix)];
                    if tgt != State::INVALID {
                        eps_set.extend(eps_mat[tgt.id()].iter());
                    }
                }
                if !eps_set.is_empty() {
                    dfa_mat[(i,voc_ix)] = match dfa_eps_map.get(&eps_set) {
                        Some(&id) => id,
                        None => {
                            let id = State::new(dfa_eps_ixs.len());
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
        for (dfa_ix,epss) in dfa_eps_ixs.into_iter().enumerate() {
            let has_name = |&key:&State| nfa.states[key.id()].name.is_some();
            if let Some(eps) = epss.into_iter().find(has_name) {
                let rule = nfa.states[eps.id()].name.as_ref().cloned().unwrap();
                callbacks[dfa_ix] = Some(RuleExecutable{code:rule,priority});
            }
        }

        let alphabet = (&nfa.alphabet).into();
        let links    = dfa_mat;
        Dfa {alphabet,links,callbacks}
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
pub mod tests {
    use crate::automata::state;

    use super::*;

    const INVALID:usize = State::INVALID.id;

    /// Dfa automata that accepts newline '\n'.
    pub fn newline() -> Dfa {
        Dfa {
            alphabet: alphabet::Segmentation::from_divisions(&[10,11]),
            links: Matrix::from(vec![vec![INVALID,1,INVALID], vec![INVALID,INVALID,INVALID]]),
            callbacks: vec![
                None,
                Some(RuleExecutable {priority:2, code:"group0_rule0".into()}),
            ],
        }
    }

    /// Dfa automata that accepts any letter a..=z.
    pub fn letter() -> Dfa {
        Dfa {
            alphabet: alphabet::Segmentation::from_divisions(&[97,123]),
            links: Matrix::from(vec![vec![INVALID,1,INVALID], vec![INVALID,INVALID,INVALID]]),
            callbacks: vec![
                None,
                Some(RuleExecutable {priority:2, code:"group0_rule0".into()}),
            ],
        }
    }

    /// Dfa automata that accepts any number of spaces ' '.
    pub fn spaces() -> Dfa {
        Dfa {
            alphabet: alphabet::Segmentation::from_divisions(&[0,32,33]),
            links: Matrix::from(vec![
                vec![INVALID,1,INVALID],
                vec![INVALID,2,INVALID],
                vec![INVALID,2,INVALID],
            ]),
            callbacks: vec![
                None,
                Some(RuleExecutable {priority:3, code:"group0_rule0".into()}),
                Some(RuleExecutable {priority:3, code:"group0_rule0".into()}),
            ],
        }
    }

    /// Dfa automata that accepts one letter a..=z or any many spaces.
    pub fn letter_and_spaces() -> Dfa {
        Dfa {
            alphabet: alphabet::Segmentation::from_divisions(&[32,33,97,123]),
            links: Matrix::from(vec![
                vec![INVALID,      1,INVALID,      2,INVALID],
                vec![INVALID,      3,INVALID,INVALID,INVALID],
                vec![INVALID,INVALID,INVALID,INVALID,INVALID],
                vec![INVALID,      3,INVALID,INVALID,INVALID],
            ]),
            callbacks: vec![
                None,
                Some(RuleExecutable {priority:4, code:"group0_rule1".into()}),
                Some(RuleExecutable {priority:4, code:"group0_rule0".into()}),
                Some(RuleExecutable {priority:4, code:"group0_rule1".into()}),
            ],
        }
    }
}
