use crate::automata::alphabet::Alphabet;
use crate::automata::state;

use std::ops::Index;
use std::ops::IndexMut;


// =====================================
// === Deterministic Finite Automata ===
// =====================================


/// Efficient 2D matrix.
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct Matrix<T> {
    /// The number of rows in matrix.
    rows: usize,
    /// The number of columns in matrix.
    columns: usize,
    /// Matrix implemented with vector.
    matrix: Vec<T>,
}

/// Function callback for an arbitrary state of finite automata.
/// It contains name of Rust procedure that is meant to be executed
/// after encountering a pattern (declared in `group::Rule.pattern`).
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Callback {
    /// TODO[jv] figure out where it is used and describe it.
    pub priority: usize,
    /// Name of Rust method that will be called when executing this callback.
    pub name: String,
}

/// DFA automata with a set of symbols, states and transitions.
/// Deterministic Finite Automata is a finite-state machine
/// that accepts or rejects a given sequence of symbols,
/// by running through a state sequence uniquely determined
/// by the input symbol sequence.
///   ___              ___              ___              ___
///  | 0 | -- 'D' --> | 1 | -- 'F' --> | 2 | -- 'A' --> | 3 |
///   ‾‾‾              ‾‾‾              ‾‾‾              ‾‾‾
/// More information at: https://en.wikipedia.org/wiki/Deterministic_finite_automaton

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct DFA {
    /// Finite set of all valid input symbols.
    pub alphabet: Alphabet,
    /// Transition matrix of deterministic finite state automata.
    /// It contains next state for each pair of state and input symbol - (state,symbol) => new state.
    /// For example, a transition matrix for automata that accepts string "ABABAB...." would look
    /// like this:
    ///  states
    /// |       | A | B | <- symbols
    /// | 0     | 1 | - |
    /// | 1     | - | 0 |
    ///  Where `-` denotes `state::INVALID`.
    pub links: Matrix<state::Id>,
    /// Stores callback for each state (if it has one).
    pub callbacks: Vec<Option<Callback>>,
}

impl<T:Default> Matrix<T> {
    /// Constructs a new matrix for given number of rows and columns.
    pub fn new(rows:usize, columns:usize) -> Self {
        let mut matrix = Vec::with_capacity(rows*columns);
        for _ in 0..matrix.capacity() {
            matrix.push(Default::default())
        }
        Self{rows,columns,matrix}
    }

    /// Adds a new row to matrix, filled with default values.
    pub fn new_row(&mut self) {
        for _ in 0..self.columns {
            self.matrix.push(Default::default());
        }
        self.rows += 1;
    }
}

impl<T> Index<(usize,usize)> for Matrix<T> {
    type Output = T;
    fn index(&self, index:(usize,usize)) -> &T {
        &self.matrix[index.0*self.columns+index.1]
    }
}

impl<T> IndexMut<(usize,usize)> for Matrix<T> {
    fn index_mut(&mut self, index:(usize,usize)) -> &mut T {
        &mut self.matrix[index.0*self.columns+index.1]
    }
}

impl From<Vec<Vec<usize>>> for Matrix<state::Id> {
    fn from(input:Vec<Vec<usize>>) -> Self {
        let rows        = input.len();
        let columns     = if rows == 0 {0} else {input[0].len()};
        let mut matrix  = Vec::<state::Id>::new();
        for row in input {
            matrix.extend(row.into_iter().map(|id| state::Id{id}))
        }
        Self {rows,columns,matrix}
    }
}


// ===========
// == Tests ==
// ===========

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::automata::state;


    const I:usize = state::INVALID.id;

    #[test]
    pub fn foo() {
        let mut m = Matrix::from(vec![vec![3,1],vec![2,2]]);
        m.new_row();
        m.new_row();
        for i in 0..4 {
            for j in 0..2 {
                println!("{:?}",m[(i,j)])
            }
        }
    }

    /// DFA automata that accepts newline '\n'.
    pub fn newline() -> DFA {
        DFA {
            alphabet: Alphabet::from(vec![10,11]),
            links: Matrix::from(vec![vec![I,1,I], vec![I,I,I]]),
            callbacks: vec![
                None,
                Some(Callback{priority:2,name:"group0_rule0".into()}),
            ],
        }
    }

    /// DFA automata that accepts any letter a..=z.
    pub fn letter() -> DFA {
        DFA {
            alphabet: Alphabet::from(vec![97,123]),
            links: Matrix::from(vec![vec![I,1,I], vec![I,I,I]]),
            callbacks: vec![
                None,
                Some(Callback{priority:2,name:"group0_rule0".into()}),
            ],
        }
    }

    /// DFA automata that accepts any number of spaces ' '.
    pub fn spaces() -> DFA {
        DFA {
            alphabet: Alphabet::from(vec![0,32,33]),
            links: Matrix::from(vec![
                vec![I,1,I],
                vec![I,2,I],
                vec![I,2,I],
            ]),
            callbacks: vec![
                None,
                Some(Callback{priority:3,name:"group0_rule0".into()}),
                Some(Callback{priority:3,name:"group0_rule0".into()}),
            ],
        }
    }

    /// DFA automata that accepts one letter a..=z or any many spaces.
    pub fn letter_and_spaces() -> DFA {
        DFA {
            alphabet: Alphabet::from(vec![32,33,97,123]),
            links: Matrix::from(vec![
                vec![I,1,I,2,I],
                vec![I,3,I,I,I],
                vec![I,I,I,I,I],
                vec![I,3,I,I,I],
            ]),
            callbacks: vec![
                None,
                Some(Callback{priority:4,name:"group0_rule1".into()}),
                Some(Callback{priority:4,name:"group0_rule0".into()}),
                Some(Callback{priority:4,name:"group0_rule1".into()}),
            ],
        }
    }
}
