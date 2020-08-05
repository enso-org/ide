//! This module defines utilities for working with the `Result` type.

/// Adds methods to the `Result` type.
#[allow(missing_docs)]
pub trait ResultOps {
    type Value;
    type Error;
    fn unwrap(self) -> Self::Value;
}

impl<V,E> ResultOps for Result<V,E> {
    type Value = V;
    type Error = E;

    fn unwrap(self) -> Self::Value {
        match self {
            Ok(v)  => v,
            Err(e) => panic!("called `Result::unwrap()` on a `Err` value."),
        }
    }
}
