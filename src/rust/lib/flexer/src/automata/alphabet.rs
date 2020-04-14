use crate::automata::state::Symbol;

use std::collections::BTreeSet;
use std::ops::Range;



/// An alphabet for an automata.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Alphabet {
    /// Set of sorted symbols.
    pub symbols: BTreeSet<Symbol>
}

impl Default for Alphabet {
    fn default() -> Self {
        Alphabet {symbols:[0].iter().cloned().collect()}
    }
}

impl Alphabet {
    /// Creates alphabet from a slice of symbols.
    pub fn new(iter:&[i64]) -> Self {
        let mut dict = Self::default();
        for &code in iter {
            dict.symbols.insert(code);
        }
        dict
    }

    /// Inserts a range of symbols into the alphabet.
    pub fn insert(&mut self, range:Range<Symbol>) {
        self.symbols.insert(range.start);
        self.symbols.insert(range.end + 1);
    }
}
