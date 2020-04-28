//! A class for monoids (types with an associative binary operation that has an identity) with
//! various general-purpose instances.

use super::semigroup::Semigroup;
use super::semigroup::SemigroupIm;



// ===============
// === Monoid ====
// ===============

/// Mutable Monoid definition.
pub trait Monoid : Default + Semigroup {
    /// Repeat a value n times. Given that this works on a Monoid it will not fail if you request 0
    /// or fewer repetitions.
    fn times_mut(&mut self, n:usize) {
        if n == 0 {
            *self = Default::default()
        } else {
            let val = self.clone();
            for _ in 0..n-1 {
                self.concat_mut(&val)
            }
        }
    }
}


/// Immutable Monoid definition.
pub trait MonoidIm : Default + SemigroupIm {
    /// Repeat a value n times. Given that this works on a Monoid it will not fail if you request 0
    /// or fewer repetitions.
    fn times(&self, n:usize) -> Self {
        vec![self].iter().cycle().take(n).fold(Default::default(),|l,r| l.concat(r))
    }
}


// === Default Impls ===

impl<T> Monoid   for T where T : Default + Semigroup   {}
impl<T> MonoidIm for T where T : Default + SemigroupIm {}
