use crate::automata::alphabet::Alphabet;
use crate::automata::state;

use std::ops::RangeInclusive;



/// Invalid state flag.
pub const INVALID:StateId = StateId{id:usize::max_value()};

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct Symbol { pub val: i64 }

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct StateId { pub id: usize }


/// NFA state with name and set of transitions (links).
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct State {
    /// Set of links that don't require any symbol to trigger.
    /// I.E. If there is an epsilon link from state A to state B,
    /// then whenever we are in state A, we can freely move to state B.
    pub epsilon_links : Vec<StateId>,
    /// Set of links that require specific symbol to trigger.
    pub links         : Vec<Link>,
    /// Name of the state.
    /// We use it to autogenerate a call to method with same name.
    pub name          : Option<String>,
}

/// A link that requires specific range of symbols to trigger.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Link {
    /// Range of symbols that trigger this link.
    pub symbols : RangeInclusive<Symbol>,
    /// A state this link points to.
    pub target  : StateId,
}

impl State {
    /// Creates a state with given epsilon links.
    pub fn epsilon_links(iter:&[usize]) -> Self {
        let link = |id| StateId{id};
        let epsilon_links = iter.iter().cloned().map(link).collect();
        State {epsilon_links, ..Default::default()}
    }

    /// Creates a state with given links.
    pub fn links(iter:&[(RangeInclusive<i64>,usize)]) -> Self {
        let link = |(range,id):(RangeInclusive<i64>,usize)| {
            let symbols = Symbol{val:*range.start()}..=Symbol{val:*range.end()};
            Link{symbols,target:StateId{id}}
        };
        let links = iter.iter().cloned().map(link).collect();
        State {links , ..Default::default()}
    }

    /// Gives state a name.
    pub fn named(mut self, name:&str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Returns target for each symbol in alphabet.
    pub fn targets(&self, alphabet:&Alphabet) -> Vec<StateId> {
        let mut targets = vec![];
        let mut index   = 0;
        let mut links   = self.links.clone();
        links.sort_by_key(|link| *link.symbols.start());
        for &symbol in &alphabet.symbols {
            while links.len() > index && *links[index].symbols.end() < symbol {
                index += 1;
            }
            if links.len() <= index || *links[index].symbols.start() > symbol {
                targets.push(state::INVALID);
            } else {
                targets.push(links[index].target);
            }
        }
        targets
    }
}