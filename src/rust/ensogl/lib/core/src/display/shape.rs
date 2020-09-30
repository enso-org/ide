//! This is a root module for shapes, 2-dimensional graphical elements.

pub mod compound;
pub mod primitive;
pub mod text;

pub use primitive::*;
// We have two Shape and two ShapeOps traits. This one takes precedence.
pub use primitive::def::class::ShapeOps;
