//! The module contains all structures for representing suggestions and their database.
//!
use crate::prelude::*;

use enso_protocol::language_server;
use language_server::types::SuggestionsDatabaseVersion;

pub use language_server::types::SuggestionEntry as Entry;
pub use language_server::types::SuggestionEntryId as EntryId;



// ================
// === Database ===
// ================

/// The Suggestion Database
///
/// This is database of possible suggestions in Searcher. To achieve best performance, some
/// often-called Language Server methods returns the list of keys of this database instead of the
/// whole entries. Additionally the database contains information about functions and their
/// argument names and types.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct SuggestionDatabase {
    entries : RefCell<HashMap<EntryId,Rc<Entry>>>,
    version : RefCell<SuggestionsDatabaseVersion>,
}

impl SuggestionDatabase {
    /// Create a new database model from response received from the Language Server.
    pub fn new_from_ls_response(response:language_server::response::GetSuggestionDatabase) -> Self {
        let mut entries = HashMap::new();
        for entry in response.entries {
            entries.insert(entry.id, Rc::new(entry.suggestion));
        }
        Self {
            entries : RefCell::new(entries),
            version : RefCell::new(response.current_version),
        }
    }

    /// Get suggestion entty by id.
    pub fn get(&self, id:EntryId) -> Rc<Entry> {
        self.entries.borrow().get(&id).clone_ref()
    }
}
