use crate::prelude::*;

use crate::list_view::entry;
use crate::list_view::Entry;



// ======================
// === Empty Provider ===
// ======================

/// An Entry Model Provider giving no entries.
///
/// This is the default provider for new select components.
#[derive(Clone,CloneRef,Copy,Debug)]
pub struct Empty;

impl<E> entry::Provider<E> for Empty {
    fn entry_count(&self)          -> usize                            { 0    }
    fn get        (&self, _:usize) -> Option<E::Model> where E : Entry { None }
}
