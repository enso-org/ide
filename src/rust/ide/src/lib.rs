//! Main library crate for IDE. It includes implementation of
//! controllers, view logic and code that wraps them all together.
//!
#![feature(arbitrary_self_types)]
#![feature(async_closure)]
#![feature(associated_type_bounds)]
#![feature(bool_to_option)]
#![feature(cell_update)]
#![feature(drain_filter)]
#![feature(exact_size_is_empty)]
#![feature(iter_order_by)]
#![feature(option_result_contains)]
#![feature(trait_alias)]
#![feature(matches_macro)]
#![recursion_limit="256"]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod config;
pub mod constants;
pub mod controller;
pub mod double_representation;
pub mod executor;
pub mod ide;
pub mod model;
pub mod notification;
pub mod test;
pub mod transport;
pub mod view;

pub use crate::ide::IdeInitializer;

use ensogl::system::web;
use wasm_bindgen::prelude::*;

#[cfg(test)]
mod tests;

/// Common types that should be visible across the whole IDE crate.
pub mod prelude {
    pub use ensogl::prelude::*;
    pub use ensogl::prelude::enabled::Logger;
    pub use enso_prelude::*;
    pub use ast::prelude::*;
    pub use wasm_bindgen::prelude::*;

    pub use crate::constants;
    pub use crate::controller;
    pub use crate::double_representation;
    pub use crate::executor;
    pub use crate::model;

    pub use enso_protocol::prelude::BoxFuture;
    pub use enso_protocol::prelude::StaticBoxFuture;
    pub use enso_protocol::prelude::StaticBoxStream;

    pub use futures::Future;
    pub use futures::FutureExt;
    pub use futures::Stream;
    pub use futures::StreamExt;
    pub use futures::task::LocalSpawnExt;

    pub use std::ops::Range;

    pub use utils::fail::FallibleResult;
    pub use utils::option::OptionExt;
    pub use utils::vec::VecExt;

    pub use uuid::Uuid;

    #[cfg(test)] pub use wasm_bindgen_test::wasm_bindgen_test;
    #[cfg(test)] pub use wasm_bindgen_test::wasm_bindgen_test_configure;
}

/// IDE startup function.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_ide() {
    web::forward_panic_hook_to_console();
    web::set_stdout();

    // FIXME: This code is temporary. It's used to remove the loader UI.
    ensogl_core_msdf_sys::run_once_initialized(|| {
        web::get_element_by_id("loader").map(|t| {
            t.parent_node().map(|p| {
                p.remove_child(&t).unwrap()
            })
        }).ok();
        IdeInitializer::new().start_and_forget();
    });
}
