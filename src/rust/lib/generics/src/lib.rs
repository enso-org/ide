
#![deny(unconditional_recursion)]

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

#![feature(const_fn)]
#![feature(specialization)]
#![feature(trait_alias)]

pub mod generic;
pub mod hlist;
pub mod tuple;

pub use generic::*;
pub use hlist::*;
pub use tuple::*;
