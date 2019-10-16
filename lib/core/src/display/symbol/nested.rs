use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::dirty;
use crate::display::symbol::attr;
use crate::display::symbol::attr::SharedAttr;
use crate::system::web::Logger;
use crate::system::web::group;
use crate::system::web::fmt;
use std::slice::SliceIndex;

// =============
// === Scope ===
// =============

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound = "Child:Debug"))]
pub struct Nested<Child, OnDirty = NoCallback> {
    pub children : Vec<Child>,
    pub dirty    : Dirty<OnDirty>,
    pub logger   : Logger,
}

// === Types ===

pub type  Index                      = usize;
pub type  Dirty         <OnDirty>    = dirty::SharedBitField<u64, OnDirty>;
pub type  OnChildChange <OnDirty>    = impl Fn();
pub trait ChildBuilder  <OnDirty, T> = FnOnce(OnChildChange<OnDirty>) -> T;

// === Implementation ===

pub fn child_on_change<OnDirty: Callback0>(
    dirty : &Dirty<OnDirty>,
    ix    : usize,
) -> OnChildChange<OnDirty> {
    let dirty = dirty.clone();
    move || dirty.set(ix)
}

impl<Child, OnDirty> Nested<Child, OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        logger.info("Initializing.");
        let dirty_logger = logger.sub("dirty");
        let dirty        = Dirty::new(on_dirty, dirty_logger);
        let children     = Vec::new();
        Self { children, dirty, logger }
    }

    pub fn child_by_ix(&self, ix: Index) -> &Child {
        &self.children[ix]
    }

    pub fn child_by_ix_mut(&mut self, ix: Index) -> &mut Child {
        &mut self.children[ix]
    }
}

impl<Child, OnDirty: Callback0> Nested<Child, OnDirty> {
    pub fn add<F: ChildBuilder<OnDirty, Child>>(&mut self, bldr: F) -> Index {
        let index = self.children.len();
        self.logger.info(fmt!("Registering at index {}.", index));
        let attr = bldr(child_on_change(&self.dirty, index));
        self.children.push(attr);
        index
    }
}


