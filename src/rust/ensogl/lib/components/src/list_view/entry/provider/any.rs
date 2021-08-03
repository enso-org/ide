use crate::prelude::*;

use crate::list_view::entry;



// ====================
// === Any Provider ===
// ====================

/// A wrapper for shared instance of some Provider of models for `E` entries.
#[derive(Debug,Shrinkwrap)]
pub struct Any<E>(Rc<dyn entry::Provider<E>>);

impl<E> Clone    for Any<E> { fn clone    (&self) -> Self { Self(self.0.clone())     }}
impl<E> CloneRef for Any<E> { fn clone_ref(&self) -> Self { Self(self.0.clone_ref()) }}

impl<E> Any<E> {
    /// Create from typed provider.
    pub fn new<T:entry::Provider<E>+'static>(provider:T) -> Self {
        Self(Rc::new(provider))
    }
}

impl<E,T:entry::Provider<E>+'static> From<Rc<T>> for Any<E> {
    fn from(provider:Rc<T>) -> Self { Self(provider) }
}

impl<E> Default for Any<E> {
    fn default() -> Self {
        Self::new(entry::provider::Empty)
    }
}
