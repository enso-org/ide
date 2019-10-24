use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::dirty;
use crate::dirty::SharedBool;
use crate::display::symbol::scope;
use crate::display::symbol::scope::Scope;
use crate::display::symbol::geometry;
use crate::system::web::Logger;
use crate::system::web::group;
use crate::system::web::fmt;
use std::slice::SliceIndex;
use crate::closure;
use paste;


// ============
// === Mesh ===
// ============

// === Definition ===

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Mesh<OnDirty> {
    #[shrinkwrap(main_field)]
    pub geometry       : Geometry      <OnDirty>,
    pub geometry_dirty : GeometryDirty <OnDirty>,
    pub logger         : Logger,
}

// === Types ===

pub type GeometryDirty <Callback> = SharedBool<Callback>;
pub type Geometry      <Callback> = geometry::Geometry
    <Closure_geometry_on_change<Callback>>;

// === Callbacks ===

closure!(geometry_on_change<Callback: Callback0>
    (dirty: GeometryDirty<Callback>) 
        || { dirty.set() });

// === Implementation ===

impl<OnDirty: Callback0> Mesh<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let geometry_logger = logger.sub("geometry_dirty");
        let geometry_dirty  = GeometryDirty::new(on_dirty, geometry_logger);
        let geo_on_change   = geometry_on_change(geometry_dirty.clone());
        let geometry        = group!(logger, "Initializing.", {
            Geometry::new(logger.sub("geometry"), geo_on_change)
        });
        Mesh { geometry, geometry_dirty, logger }
    }
}



