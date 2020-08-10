//! A module containing a filtered and ordered list of searcher suggestions.
use crate::prelude::*;

use crate::controller::searcher;



// =============================
// === Suggestion List Entry ===
// =============================

/// Suggestion for input completion: possible functions, arguments, etc.
pub type CompletionSuggestion = Rc<model::suggestion_database::Entry>;

/// A single suggestion on the Searcher suggestion list.
#[derive(Clone,CloneRef,Debug,Eq,PartialEq)]
pub enum Suggestion {
    /// Suggestion for input completion: possible functions, arguments, etc.
    Completion(CompletionSuggestion)
    // In future, other suggestion types will be added (like suggestions of actions, etc.).
}

impl Suggestion {
    pub fn name(&self) -> &String {
        match self {
            Self::Completion(completion) => &completion.name
        }
    }
}

pub enum QueryScore {
    FilteredOut,
    FilteredIn{score:f32}
}

pub struct Entry {
    score      : QueryScore,
    suggestion : Suggestion,
}



// =======================
// === Suggestion List ===
// =======================

pub struct List {
    entries : RefCell<Vec<Entry>>
}

impl List {
    fn new()
}



