#![feature(type_ascription)]
#![feature(unboxed_closures)]
#![cfg_attr(test, allow(dead_code))]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
//#![warn(missing_docs)]

// Lints. To be refactored after this gets resolved: https://github.com/rust-lang/cargo/issues/5034
#![allow(clippy::option_map_unit_fn)]

// TODO: remove unstable features unless one will get stabilized soon

// =================================
// === Module Structure Reexport ===
// =================================

pub mod data;
pub mod dirty;
pub mod display;
pub use basegl_prelude as prelude;
pub mod backend {
    pub use basegl_backend_webgl as webgl;
}
pub mod system {
    pub use basegl_system_web as web;
}

// ============
// === Main ===
// ============

use display::world::World;
use wasm_bindgen::prelude::*;

use display::symbol::attr::SharedAttr;
use system::web::Logger;
use system::web::fmt;

use bit_field::BitField;
use crate::display::symbol::scope::Scope;
use crate::display::symbol::attr;
use crate::display::symbol::attr::Attr;

#[wasm_bindgen(start)]
pub fn start() {
    let world = World::new();
    world.add_workspace("canvas");
    world.start();

//    let logger = Logger::new("test");
//
//    let pos: attribute::Builder<f32> = Attribute::builder();
//
//    let pos: SharedAttribute<f32> = SharedAttribute::new(logger,());

    let logger = Logger::new("point");
    let mut point_scope: Scope = Scope::new(logger,());
    point_scope.add("position", Attr::builder());

//    let logger = Logger::new("local");
//
//    logger.info(fmt!("{:#?}",point_scope));
}
