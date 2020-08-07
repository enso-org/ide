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
    fn name(&self) -> &String {
        match self {
            Self::Completion(completion) => &completion.name
        }
    }

    fn matches(&self, query:impl Str) -> bool {
        let mut query_chars     = query.as_ref().chars();
        let mut next_query_char = query_chars.next();
        for name_char in self.name().chars() {
            if let Some(query_char) = next_query_char {
                if query_char.to_lowercase() == name_char.to_lowercase() {
                    next_query_char = query_char.next()
                }
            } else {
                break;
            }
        }
        next_query_char.is_none()
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

}



// ===============
// ===  ===
// ===============

struct SubsequenceGraph {
    data : nalgebra::DMatrix<usize>,
}

impl SubsequenceGraph {
    fn create_for(word:impl Str, query:impl Str) -> Self {
        let mut data = nalgebra::DMatrix::<usize>::new();
        data.resize(query.as_ref().chars().count(),word.as_ref().chars_count());

    }
}
