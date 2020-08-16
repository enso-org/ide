//! Defines a Symbol that is operated on by the finite automata.

use crate::prelude::*;



// ==============
// === Symbol ===
// ==============

/// An input symbol to a finite automaton.
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[allow(missing_docs)]
pub struct Symbol {
    pub index: u64
}

impl Symbol {
    /// A representation of the end of the file.
    pub const EOF : Symbol = Symbol::new(u64::max_value());

    /// A representation of the null symbol.
    pub const NULL : Symbol = Symbol::new(0);
}

impl Symbol {
    /// Constructor.
    pub const fn new(index:u64) -> Self {
        Self {index}
    }

    /// Next symbol, if any.
    pub fn next(self) -> Option<Self> {
        (self.index < u64::max_value() - 1).as_some_from(|| {
            Self::new(self.index + 1)
        })
    }
}


// === Impls ===

impl Default for Symbol {
    fn default() -> Self {
        Symbol::NULL
    }
}

impl From<u64> for Symbol {
    fn from(index:u64) -> Symbol {
        Symbol::new(index)
    }
}

impl From<char> for Symbol {
    fn from(ch:char) -> Symbol {
        Symbol::new(ch as u64)
    }
}

impl From<&Symbol> for Symbol {
    fn from(symbol:&Symbol) -> Self {
        *symbol
    }
}
