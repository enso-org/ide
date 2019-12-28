#![cfg_attr(test, allow(dead_code))]
#![feature(unboxed_closures)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(specialization)]
#![feature(associated_type_defaults)]
#![deny(missing_docs)]

// To be removed after this gets resolved: https://github.com/rust-lang/cargo/issues/5034
#![allow(clippy::option_map_unit_fn)]

//! BaseGL is a blazing fast 2D vector rendering engine with a rich set of primitives and a GUI
//! component library. It is able to display millions of shapes 60 frames per second in a web
//! browser on a modern laptop hardware. This is the main entry point to the library.


// ===================
// === Macro Debug ===
// ===================

//#![feature(trace_macros)]
//#![recursion_limit="256"]
//trace_macros!(true);


// =================================
// === Module Structure Reexport ===
// =================================

pub mod animation;
pub mod control;
pub mod data;
pub mod debug;
pub mod display;
pub mod examples;
pub mod traits;

pub use basegl_prelude as prelude;
/// System module.
pub mod system {
    pub use basegl_system_web as web;
}


