use crate::automata::alphabet::Alphabet;

use std::ops::Range;
use crate::automata::state;


pub const MISSING:usize = usize::max_value();

pub type Symbol  = i64;
pub type StateId = usize;


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
    pub symbols : Range<Symbol>,
    /// A state this link points to.
    pub target  : StateId,
}

impl State {
    /// Creates a state with given epsilon links.
    pub fn epsilon_links(iter:&[StateId]) -> Self {
        State { epsilon_links: iter.iter().cloned().collect(), ..Default::default() }
    }

    /// Creates a state with given links.
    pub fn links(iter:&[(Range<Symbol>,StateId)]) -> Self {
        let link = |(symbols,target)| Link{symbols,target};
        State { links: iter.iter().cloned().map(link).collect(), ..Default::default() }
    }

    /// Gives state a name.
    pub fn named(mut self, name:&str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Returns target for each symbol in alphabet.
    pub fn targets(&self, alphabet:&Alphabet) -> Vec<usize> {
        let mut targets = vec![];
        let mut index   = 0;
        let mut links   = self.links.clone();
        links.sort_by_key(|link| link.symbols.start);
        for &symbol in &alphabet.symbols {
            while links.len() > index && links[index].symbols.end < symbol {
                index += 1;
            }
            if links.len() <= index || links[index].symbols.start > symbol {
                targets.push(state::MISSING);
            } else {
                targets.push(links[index].target);
            }
        }
        targets
    }
}