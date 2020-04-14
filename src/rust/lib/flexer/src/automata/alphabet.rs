use crate::automata::state::Symbol;

use std::collections::BTreeSet;
use std::ops::RangeInclusive;



/// An alphabet for an automata.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Alphabet {
    /// Set of sorted symbols.
    pub symbols: BTreeSet<Symbol>
}

impl Default for Alphabet {
    fn default() -> Self {
        Alphabet {symbols:[Symbol{val:0}].iter().cloned().collect()}
    }
}

impl Alphabet {
    /// Creates alphabet from a slice of symbols.
    pub fn new(iter:&[i64]) -> Self {
        let mut dict = Self::default();
        for &val in iter {
            dict.symbols.insert(Symbol{val});
        }
        dict
    }

    /// Inserts a range of symbols into the alphabet.
    pub fn insert(&mut self, range:RangeInclusive<Symbol>) {
        self.symbols.insert(Symbol{val:range.start().val});
        self.symbols.insert(Symbol{val:range.end().val + 1});
    }
}
