//! Client side implementation of Enso protocol.

#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
//#![feature(async_closure)]
#![feature(associated_type_bounds)]
#![feature(associated_type_defaults)]
#![feature(coerce_unsized)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod binary;
pub mod common;
pub mod generated;
pub mod types;
pub mod language_server;
pub mod project_manager;
pub mod new_handler;

pub mod prelude {
    pub use json_rpc::prelude::*;
    pub use utils::fail::FallibleResult;
    pub use uuid::Uuid;

    pub use crate::traits::*;
    pub use logger::*;

    pub use std::future::Future;
    pub use futures::FutureExt;
    pub use futures::StreamExt;
}

/// Module gathering all traits which may be used by crate's users.
pub mod traits {
    pub use crate::language_server::API as TRAIT_LanguageServerAPI;
    pub use crate::project_manager::API as TRAIT_ProjectManagerAPI;
    pub use crate::binary::uuid::UuidExt;
    pub use crate::binary::API;
}

