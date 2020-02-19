//! Main library crate for IDE. It includes implementation of
//! controllers, view logic and code that wraps them all together.

#![feature(trait_alias)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod controller;
#[allow(unused)]
pub mod entry_point;
pub mod executor;
pub mod transport;
pub mod view;

#[allow(missing_docs)]
/// Common types that should be visible across the whole IDE crate.
pub mod prelude {
    pub use enso_prelude::*;

    pub use crate::controller;
    pub use crate::executor;

    pub use futures::Future;
    pub use futures::FutureExt;
    pub use futures::Stream;
    pub use futures::StreamExt;
    pub use futures::task::LocalSpawnExt;
    pub use wasm_bindgen::prelude::*;
}
