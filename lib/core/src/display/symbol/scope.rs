use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::dirty;
use crate::display::symbol::attr;
use crate::display::symbol::attr::SharedAttr;
use crate::system::web::Logger;
use crate::system::web::group;
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
type Attribute    <OnDirty> = SharedAttr<f32, OnAttrChange<OnDirty>>;
type AttrReg      <OnDirty> = Vec<Attribute<OnDirty>>;
type OnAttrChange <OnDirty> = impl Fn();

// === Implementation ===

fn attr_on_change<OnDirty: Callback0>(dirty: &Dirty<OnDirty>, ix: usize) -> OnAttrChange<OnDirty> {
    let dirty = dirty.clone();
    move || dirty.set(ix)
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
    pub fn add<Name: AsRef<str>>(&mut self, name: Name, attr_bldr: attr::Builder<f32>) -> AttrIndex
    {
        let name  = name.as_ref().to_string();
        let index = self.attrs.len();
        group!(self.logger, format!("Adding attribute '{}' at index {}.", name, index), {
            let attr_bldr = attr_bldr.logger(self.logger.sub(&name));
            let attr      = Attribute::build(attr_bldr, attr_on_change(&self.dirty, index));
            self.attrs.push(attr);
            self.name_map.insert(name, index);
            index
        })
    }
}

//type Closure = impl Fn(i32);
//fn mk_closure() -> Closure {
//    move |i| {}
//}
//
//
//
//pub fn test() -> i32
//    where Closure: FnMut(i8) {
//    5
//}
//


