use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::dirty;
use crate::display::symbol::attribute;
use crate::display::symbol::attribute::SharedAttribute;
use crate::system::web::Logger;
use crate::system::web::fmt;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

// =============
// === Scope ===
// =============

// === Definition ===

#[derive(Debug)]
pub struct Scope <OnDirty = NoCallback> {
    pub attrs    : AttrReg <OnDirty>,
    pub name_map : HashMap <AttrName, AttrIndex>,
    pub dirty    : Dirty   <OnDirty>,
    pub logger   : Logger,
}

// === Types ===

type AttrName               = String;
type AttrIndex              = usize;
type Dirty        <OnDirty> = dirty::SharedBitField<u64, OnDirty>;
type Attribute    <OnDirty> = SharedAttribute<f32, OnAttrChange<OnDirty>>;
type AttrReg      <OnDirty> = Vec<Attribute<OnDirty>>;
type OnAttrChange <OnDirty> = impl Fn(usize);

// === Implementation ===

fn buffer_on_change<OnDirty: Callback0>(dirty: &Dirty<OnDirty>) -> OnAttrChange<OnDirty> {
    let dirty = dirty.clone();
    move |ix| dirty.set(ix)
}

impl<OnDirty> Scope<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        logger.info("Creating new scope.");
        let dirty_logger = logger.sub("dirty");
        let dirty        = Dirty::new(on_dirty, dirty_logger);
        let name_map     = HashMap::new();
        let attrs        = Vec::new();
        Self { attrs, name_map, dirty, logger }
    }
}

impl<OnDirty: Callback0> Scope<OnDirty> {
    pub fn add<Name: AsRef<str>>(&mut self, name: Name, attr: attribute::Builder<f32>) -> AttrIndex
    where OnAttrChange<OnDirty>: Callback0 {
        let name  = name.as_ref().to_string();
        let attr  = Attribute::build(attr, buffer_on_change(&self.dirty));
        let index = self.attrs.len();
        self.logger.info(fmt!("Adding attribute '{}' at index {}.", name, index));
        self.attrs.push(attr);
        self.name_map.insert(name, index);
        index
    }
}
