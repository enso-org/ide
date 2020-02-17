//! This module is the root module for all primitive shapes and shape transform definitions.

pub mod sdf;
pub mod class;
pub mod transform;
pub mod var;

pub mod export {
    pub use super::var::*;
    pub use super::class::Shape;
    pub use super::sdf::*;
    pub use super::transform::immutable::*;
}

pub use export::*;
