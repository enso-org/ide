//! Root module for math-related utilities.

pub mod algebra;
pub mod topology;
pub mod geometry;

pub use algebra::*;

/// Common types.
pub mod types {
    pub use super::algebra::*;
}
