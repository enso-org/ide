//! SpanTree module
//!
//! SpanTree is astructure describing expression with nodes mapped to expression text spans. It can
//! be be considered a layer over AST, that add an information about chains (you can
//! iterate over all elements of infix chain like `1 + 2 + 3` or prefix chain like `foo bar baz`),
//! and provides interface for AST operations like set node to a new AST or add new element to
//! operator chain.

#![feature(associated_type_bounds)]
#![feature(trait_alias)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod action;
pub mod generate;
pub mod iter;
pub mod tree;
#[cfg(test)]
pub mod builder;

pub use tree::Node;
pub use tree::NodeRef;
pub use tree::Type;
pub use tree::SpanTree;

pub mod prelude {
    pub use enso_prelude::*;
    pub use utils::fail::FallibleResult;
}
