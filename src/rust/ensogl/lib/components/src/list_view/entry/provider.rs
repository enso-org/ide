pub mod any;
pub mod empty;
pub mod single_masked;

pub use any::Any;
pub use empty::Empty;
pub use single_masked::SingleMasked;

use crate::prelude::*;

use crate::list_view::Entry;
use crate::list_view::entry;



// =================
// === Providers ===
// =================

// === The Trait ===

/// The Model Provider for ListView's entries of type `E`.
///
/// The [`crate::ListView`] component does not display all entries at once, instead it lazily ask
/// for models of entries when they're about to be displayed. So setting the select content is
/// essentially providing an implementor of this trait.
pub trait Provider<E> : Debug {
    /// Number of all entries.
    fn len(&self) -> usize;

    /// Get the model of entry with given id. The implementors should return `None` only when
    /// requested id greater or equal to entries count.
    fn get(&self, id:entry::Id) -> Option<E::Model>
        where E : Entry;
}



// =================
// === Std Impls ===
// =================

impl<E,T> Provider<E> for Vec<T>
    where E : Entry,
          T : Debug + Clone + Into<E::Model> {
    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, id:usize) -> Option<E::Model> {
        Some(<[T]>::get(self, id)?.clone().into())
    }
}
