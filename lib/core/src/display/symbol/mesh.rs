use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::dirty;
use crate::dirty::traits::*;
use crate::display::symbol::scope;
use crate::display::symbol::geometry;
use crate::system::web::Logger;
use crate::system::web::group;
use crate::system::web::fmt;
use std::slice::SliceIndex;
use crate::closure;
use paste;
use crate::{promote, promote_all, promote_geometry_types};
use eval_tt::*;

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

pub type GeometryDirty<Callback> = dirty::SharedBool<Callback>;

promote_geometry_types!{ [OnGeometryChange] geometry }
#[macro_export]
macro_rules! promote_mesh_types { ($($args:tt)*) => {
    crate::promote_geometry_types! { $($args)* }
    promote! { $($args)* [Mesh] }
};}

// === Callbacks ===

closure! {
fn geometry_on_change<C:Callback0>(dirty:GeometryDirty<C>) ->
    OnGeometryChange { || dirty.set() }
}

// === Implementation ===

impl<OnDirty: Callback0> Mesh<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let geometry_logger = logger.sub("geometry_dirty");
        let geometry_dirty  = GeometryDirty::new(geometry_logger, on_dirty);
        let geo_on_change   = geometry_on_change(geometry_dirty.clone());
        let geometry        = group!(logger, "Initializing.", {
            Geometry::new(logger.sub("geometry"), geo_on_change)
        });
        Mesh { geometry, geometry_dirty, logger }
    }

    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            if self.geometry_dirty.check_and_unset() {
                self.geometry.update()
            }
        })
    }
}

// ==================
// === SharedMesh ===
// ==================

#[derive(Shrinkwrap)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct SharedMesh<OnDirty> {
    pub raw: RefCell<Mesh<OnDirty>>
}

impl<OnDirty: Callback0> SharedMesh<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let raw = RefCell::new(Mesh::new(logger, on_dirty));
        Self { raw }
    }
}

