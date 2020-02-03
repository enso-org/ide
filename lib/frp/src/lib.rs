//! This module implements an Functional Reactive Programming system. It is an advanced event
//! handling framework which allows describing events and actions by creating declarative event
//! flow diagrams.

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

#![feature(specialization)]
#![feature(trait_alias)]
#![feature(weak_into_raw)]
#![feature(associated_type_defaults)]

pub mod data;
pub mod debug;
pub mod io;
pub mod macros;
pub mod node;
pub mod nodes;

pub use data::*;
pub use debug::*;
pub use io::*;
pub use macros::*;
pub use node::*;
pub use nodes::*;

use enso_prelude      as prelude;
use basegl_system_web as web;
