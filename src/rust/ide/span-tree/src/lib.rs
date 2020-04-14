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

pub use tree::Node;
pub use tree::NodeRef;
pub use tree::NodeType;
pub use tree::SpanTree;

pub mod prelude {
    pub use enso_prelude::*;
    pub use utils::fail::FallibleResult;
}
