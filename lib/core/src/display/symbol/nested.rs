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

#[derive(Debug)]
pub struct Nested<Child, OnDirty = NoCallback> {
    pub children : Vec      <Child>,
    pub dirty    : Dirty    <OnDirty>,
    pub logger   : Logger,
}

// === Types ===

type Index                   = usize;
type Dirty         <OnDirty> = dirty::SharedBitField<u64, OnDirty>;
type OnChildChange <OnDirty> = impl Fn();

// === Implementation ===

fn child_on_change<OnDirty: Callback0>(dirty: &Dirty<OnDirty>, ix: usize) -> OnChildChange<OnDirty> {
    let dirty = dirty.clone();
    move || dirty.set(ix)
}

impl<Child, OnDirty> Nested<Child, OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let dirty_logger = logger.sub("dirty");
        let dirty        = Dirty::new(on_dirty, dirty_logger);
        let children     = Vec::new();
        Self { children, dirty, logger }
    }
}




