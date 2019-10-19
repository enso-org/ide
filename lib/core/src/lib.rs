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

use display::symbol::attribute::SharedAttribute;
use system::web::Logger;
use system::web::fmt;

use bit_field::BitField;
use crate::display::symbol::scope::Scope;
use crate::display::symbol::attribute;
use crate::display::symbol::attribute::Attribute;
use crate::display::symbol::geometry::Geo;
use nalgebra;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use nalgebra::Matrix;
use nalgebra::base::dimension::U1;
use nalgebra::base::dimension::U2;



#[wasm_bindgen(start)]
pub fn start() {
    let world = World::new();
    world.add_workspace("canvas");
    world.start();

//    let logger = Logger::new("test");
//
//    let pos: attribute::Builder<f32> = Attribute::builder();
//
//    let pos: SharedAttributeibute<f32> = SharedAttributeibute::new(logger,());

    // let logger = Logger::new("point");
    // let mut point_scope: Scope = Scope::new(logger,());
    // point_scope.add("position", Attr::builder());

    let logger = Logger::new("geo1");
    let mut geo1 = Geo::new(logger, ());

    let position: attribute::SharedAttribute<Vector2<f32>, _> = geo1.scopes.point.add_attribute("position", Attribute::builder());
    geo1.scopes.point.add_instance();

    let v = nalgebra::Vector3::new(0,0,0);



    let logger = Logger::new("root");

    let a = 1;
    let b = 2;
    let c = 3;

    // geo1.scopes.point

//    let logger = Logger::new("local");
//
//    logger.info(fmt!("{:#?}",position.data.borrow().index(0)));
    let mut v: Vec<f32> = vec![0.0,1.0,2.0,3.0];
    let m6: Vector2<f32> = Vector2::from_iterator(v);
    let m7: Matrix<f32, U2, U1, nalgebra::ArrayStorage<f32, U2, U1>> = m6;
    // v[0] = 7.0;
}

