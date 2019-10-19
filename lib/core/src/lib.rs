#![feature(type_ascription)]
#![feature(unboxed_closures)]
#![cfg_attr(test, allow(dead_code))]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(proc_macro_hygiene)]
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
use crate::display::symbol::geo::Geo;
use nalgebra;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;

macro_rules! cartesian_impl {
    ($out:tt [] $b:tt $init_b:tt) => {
        $out
    };
    ($out:tt [$a:ident, $($at:tt)*] [] $init_b:tt) => {
        cartesian_impl!{$out [$($at)*] $init_b $init_b}
    };
    ([$($out:tt)*] [$a:ident, $($at:tt)*] [$b:ident, $($bt:tt)*] $init_b:tt) => {
        cartesian_impl!{[$($out)* ($a, $b),] [$a, $($at)*] [$($bt)*] $init_b}
    };
}

macro_rules! cartesian {
    ([$($a:tt)*], [$($b:tt)*]) => {
        cartesian_impl!{[] [$($a)*,] [$($b)*,] [$($b)*,]}
    };
}


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

    // let logger = Logger::new("point");
    // let mut point_scope: Scope = Scope::new(logger,());
    // point_scope.add("position", Attr::builder());

    let logger = Logger::new("geo1");
    let mut geo1 = Geo::new(logger, ());

    let position: attr::SharedAttr<Vector2<f32>, _> = geo1.scopes.point.add_attribute("position", Attr::builder());
    geo1.scopes.point.add_instance();

    let v = nalgebra::Vector3::new(0,0,0);

    let logger = Logger::new("root");

    let a = 1;
    let b = 2;
    let c = 3;
    logger.info(||format!("{:?}", cartesian!([a],[b,c])));

    // geo1.scopes.point

//    let logger = Logger::new("local");
//
//    logger.info(fmt!("{:#?}",point_scope));
}
