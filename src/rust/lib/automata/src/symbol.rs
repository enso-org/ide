//! Defines a Symbol that is operated on by the finite automata.

use crate::prelude::*;



// ==============
// === Symbol ===
// ==============

/// An input symbol to a finite automaton.
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[allow(missing_docs)]
pub struct Symbol {
    pub val: u32
}

impl Symbol {
    /// A representation of the end of the file.
    pub const EOF : Symbol = Symbol::new(u32::max_value());

    /// A representation of the null symbol.
    pub const NULL : Symbol = Symbol::new(0);

    /// Constructor.
    pub const fn new(val:u32) -> Self {
        Self {val}
    }

    /// Next symbol, if any.
    pub fn next(self) -> Option<Self> {
        (self.val < u32::max_value() - 1).as_some_from(|| {
            Self { val : self.val + 1 }
        })
    }
}


// === Impls ===

impl Default for Symbol {
    fn default() -> Self {
        Symbol::NULL
    }
}

impl From<u32> for Symbol {
    fn from(val:u32) -> Symbol {
        Symbol{val}
    }
}

impl From<char> for Symbol {
    fn from(val:char) -> Symbol {
        Symbol{val:val as u32}
    }
}

impl From<&Symbol> for Symbol {
    fn from(symbol:&Symbol) -> Self {
        *symbol
    }
}
