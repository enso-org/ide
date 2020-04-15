//! General purpose functions to be reused between components, not belonging to
//! any other crate and yet not worth of being split into their own creates.

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub use enso_prelude as prelude;

pub mod channel;
pub mod env;
pub mod fail;
pub mod option;
pub mod test;
pub mod vec;
