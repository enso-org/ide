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
use crate::display::symbol::nested::Nested;
use crate::display::symbol::nested::OnChildChange;
use crate::display::symbol::nested;

// =============
// === Scope ===
// =============

// === Definition ===

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scope <OnDirty = NoCallback> {
    #[shrinkwrap(main_field)]
    pub reg      : Nested  <Attr<OnDirty>, OnDirty>,
    pub name_map : HashMap <AttrName, AttrIndex>,
}

// === Types ===

type AttrName      = String;
type AttrIndex     = nested::Index;
type AttrBlr       = attr::Builder<f32>;
type Attr<OnDirty> = SharedAttr<f32, OnChildChange<OnDirty>>;

// === Implementation ===

impl<OnDirty> Scope<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let reg      = Nested::new(logger, on_dirty);
        let name_map = default();
        Self { reg, name_map }
    }
}

impl<OnDirty: Callback0> Scope<OnDirty> {
    pub fn add<Name: Str>(&mut self, name: Name, bldr: AttrBlr) -> AttrIndex {
        let name = name.as_ref().to_string();
        let bldr = bldr.logger(self.logger.sub(&name));
        group!(self.logger, format!("Adding attribute '{}'", name), {
            self.reg.add(|callback| Attr::build(bldr, callback))
        })
    }
}
