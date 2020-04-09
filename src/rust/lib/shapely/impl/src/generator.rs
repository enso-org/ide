//! Helper code meant to be used by the code generated through usage of macros
//! from `shapely-macros` crate.

pub use shapely_macros::*;

use derivative::Derivative;
use std::ops::Generator;
use std::ops::GeneratorState;
use std::pin::Pin;
use std::marker::PhantomData;



// ==========================
// === GeneratingIterator ===
// ==========================

/// Iterates over values yielded from the wrapped `Generator`.
#[derive(Debug)]
pub struct GeneratingIterator<G: Generator>(pub G);

impl<G> Iterator for GeneratingIterator<G>
where G: Generator<Return = ()> + Unpin {
    type Item = G::Yield;
    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.0).resume() {
            GeneratorState::Yielded(element) => Some(element),
            _                                => None,
        }
    }
}



// =====================
// === EmptyIterator ===
// =====================

/// An `Iterator` type that yields no values of the given type `T`.
#[derive(Derivative)]
#[derivative(Debug,Default(bound=""))]
pub struct EmptyIterator<T>(PhantomData<T>);

impl<T> EmptyIterator<T> {
    /// Create a new empty iterator.
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T> Iterator for EmptyIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_iterator_works_for_any_type() {
        for elem in EmptyIterator::new() {
            elem: i32;
        }
        for elem in EmptyIterator::new() {
            elem: String;
        }
    }

    #[test]
    fn generating_iterator_works() {
        let generator = || {
            yield 0;
            yield 1;
            yield 2;
        };
        let expected_numbers         = vec!(0, 1, 2);
        let generator_iter           = GeneratingIterator(generator);
        let collected_result: Vec<_> = generator_iter.collect();
        assert_eq!(collected_result, expected_numbers);
    }
}
