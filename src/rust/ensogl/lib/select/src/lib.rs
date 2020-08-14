//! Selection List Component

#![feature(option_result_contains)]
#![feature(trait_alias)]
#![recursion_limit="256"]

pub mod component;
pub mod entry;

/// Commonly used types and functions.
pub mod prelude {
    pub use ensogl_core::prelude::*;
}
