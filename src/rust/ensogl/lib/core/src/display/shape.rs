//! This is a root module for shapes, 2-dimensional graphical elements.

pub mod compound;
pub mod primitive;
pub mod text;

pub use primitive::*;
//TODO[ao] we have two Shape traits and two ShapeOps traits. Which should take precedence?
pub use primitive::def::class::ShapeOps;