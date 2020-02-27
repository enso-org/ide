//! This module defines helpers and utilities for working with references.

// ============
// === With ===
// ============

/// Surprisingly useful function. Consider the following code:
///
/// ```compile_fail
/// fn init(self) -> Self {
///    let mut data = self.borrow_mut();
///    ...
///    self
///    }
/// ```
///
/// It may not compile telling that the last line moves self out, however,
/// borrow might be used there, when `data` is dropped and runs the destructor.
///
/// We can use this function to narrow-down the lifetimes. The following code
/// compiles just fine:
///
/// ```compile_fail
/// fn init(self) -> Self {
///    with(self.borrow_mut(), |mut data| {
///        ...
///    });
///    self
///    }
/// ```
pub fn with<T, F: FnOnce(T) -> Out, Out>(t: T, f: F) -> Out { f(t) }



// =============
// === ToRef ===
// =============

/// Similar to `AsRef` but more specific and automatically implemented for every type. Allows for
/// conversion `&T` to `&T` (identity) and `T` to `&T` for any type `T`. In contrast, `AsRef`
/// requires explicit impls, so for example you cannot do `let t:&() = ().as_ref()`
pub trait ToRef<T>        where T:?Sized { fn to_ref(&self) -> &T; }
impl<T>   ToRef<T> for  T where T:?Sized { fn to_ref(&self) -> &T { self } }
impl<T>   ToRef<T> for &T where T:?Sized { fn to_ref(&self) -> &T { self } }
