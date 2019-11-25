use crate::prelude::*;

use super::data::{Data, SharedData};
use crate::data::function::callback::Callback1;

// ===========
// === Var ===
// ===========

/// View for a particular buffer. Allows reading and writing buffer data
/// via the internal mutability pattern. It is implemented as a view on
/// a selected `SharedBuffer` element under the hood.
pub struct Var<T,OnSet,OnResize> {
    index  : usize,
    buffer : SharedData<T,OnSet,OnResize>
}

impl<T,OnSet,OnResize>
Var<T,OnSet,OnResize> {
    /// Creates a new variable as an indexed view over provided buffer.
    pub fn new(index:usize, buffer: SharedData<T,OnSet,OnResize>) -> Self {
        Self {index, buffer}
    }

    /// Gets immutable reference to the underlying data.
    // [1] Please refer to `Prelude::drop_lifetime` docs to learn why it is safe
    // to use it here.
    pub fn get(&self) -> IndexGuard<Data<T,OnSet,OnResize>> {
        let _borrow = self.buffer.borrow();
        let target  = _borrow.index(self.index);
        let target  = unsafe { drop_lifetime(target) }; // [1]
        IndexGuard {target,_borrow}
    }
}

impl<T,OnSet:Callback1<usize>,OnResize>
Var<T,OnSet,OnResize> {

    /// Gets mutable reference to the underlying data.
    // [1] Please refer to `Prelude::drop_lifetime` docs to learn why it is safe
    // to use it here.
    pub fn get_mut(&self) -> IndexGuardMut<Data<T,OnSet,OnResize>> {
        let mut _borrow = self.buffer.borrow_mut();
        let target      = _borrow.index_mut(self.index);
        let target      = unsafe { drop_lifetime_mut(target) }; // [1]
        IndexGuardMut {target,_borrow}
    }

    /// Modifies the underlying data by using the provided function.
    pub fn modify<F: FnOnce(&mut T)>(&self, f:F) {
        f(&mut self.buffer.borrow_mut()[self.index]);
    }
}

#[derive(Shrinkwrap)]
pub struct IndexGuard<'t,T> where
    T:Index<usize> {
    #[shrinkwrap(main_field)]
    pub target : &'t <T as Index<usize>>::Output,
    _borrow    : Ref<'t,T>
}

#[derive(Shrinkwrap)]
pub struct IndexGuardMut<'t,T> where
    T:Index<usize> {
    #[shrinkwrap(main_field)]
    pub target : &'t mut <T as Index<usize>>::Output,
    _borrow    : RefMut<'t,T>
}
