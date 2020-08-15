//! This module exports State implementation for Nondeterministic Finite Automata.

use crate::alphabet;
use crate::symbol::Symbol;

use crate::prelude::*;



// ===========
// == State ==
// ===========

/// A state identifier for an arbitrary finite automaton.
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[allow(missing_docs)]
pub struct State {
    id : usize
}

impl State {
    /// An identifier representing the invalid state.
    ///
    /// When in an invalid state, a finite automaton will reject the sequence of input symbols.
    pub const INVALID : State = Self::new(usize::max_value());
}

impl State {
    /// Constructor. Not exposed to public as it should never be possible to construct a state
    /// from a number.
    pub const fn new(id:usize) -> Self {
        Self {id}
    }

    /// Identifier of this state expressed as `usize`.
    pub fn id(&self) -> usize {
        self.id
    }
}

// === Trait Impls ===

impl Default for State {
    /// Returns state::INVALID. This is because every finite automata has an invalid state
    /// and because all transitions in automata transition matrix lead to invalid state by default.
    fn default() -> Self {
        State::INVALID
    }
}

impl Debug for State {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        let name = if *self == Self::INVALID { "INVALID".into() } else { format!("{:?}",self.id) };
        write!(f,"State({})",name)
    }
}



// ==========
// == Data ==
// ==========

/// A named state for a [`super::nfa::NFA`].
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct Data {
    /// A set of transitions that can trigger without consuming a symbol (ε-transitions).
    pub epsilon_links: Vec<State>,
    /// The set of transitions that trigger while consuming a specific symbol.
    ///
    /// When triggered, the automaton will transition to the [`Transition::target_state`].
    pub links: Vec<Transition>,
    /// The name of the state.
    ///
    /// This is used to auto-generate a call to the rust method of the same name.
    pub name: Option<String>,
}

impl Data {
    /// Updater for field `name`. Returns updated state.
    pub fn named(mut self, name:&str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Returns transition (next state) for each symbol in alphabet.
    pub fn targets(&self, alphabet:&alphabet::Segmentation) -> Vec<State> {
        let mut targets = vec![];
        let mut index   = 0;
        let mut links   = self.links.clone();
        links.sort_by_key(|link| *link.symbols.start());
        for &symbol in &alphabet.divisions {
            while links.len() > index && *links[index].symbols.end() < symbol {
                index += 1;
            }
            if links.len() <= index || *links[index].symbols.start() > symbol {
                targets.push(State::INVALID);
            } else {
                targets.push(links[index].target);
            }
        }
        targets
    }
}


// === Trait Impls ====

impl From<Vec<usize>> for Data {
    /// Creates a state with epsilon links.
    fn from(vec:Vec<usize>) -> Self {
        let epsilon_links = vec.iter().cloned().map(|id| State {id}).collect();
        Data {epsilon_links,..Default::default()}
    }
}

impl From<Vec<(RangeInclusive<u32>, usize)>> for Data {
    /// Creates a state with ordinary links.
    fn from(vec:Vec<(RangeInclusive<u32>, usize)>) -> Self {
        let link = |(range, id): (RangeInclusive<u32>, usize)| {
            let start = Symbol::new(*range.start());
            let end   = Symbol::new(*range.end());
            Transition::new(start..=end, State::new(id))
        };
        let links = vec.iter().cloned().map(link).collect();
        Data {links,..Default::default()}
    }
}



// ==================
// === Transition ===
// ==================

/// A transition between states in a finite automaton that must consume a symbol to trigger.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Transition {
    /// The range of symbols on which this transition will trigger.
    pub symbols: RangeInclusive<Symbol>,
    /// The state that is entered after the transition has triggered.
    pub target: State,
}

impl Transition {
    /// Constructor.
    pub fn new(symbols:RangeInclusive<Symbol>, target:State) -> Self {
        Self {symbols,target}
    }
}