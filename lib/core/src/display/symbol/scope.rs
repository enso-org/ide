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
use crate::display::symbol::nested::Seq;
use crate::display::symbol::nested::OnChildChange;
use crate::display::symbol::nested;

// =============
// === Scope ===
// =============

// === Definition ===

// #[derive(Shrinkwrap)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scope <OnDirty = NoCallback> {
    pub seq      : Seq <Attr<OnDirty>, OnDirty>,
    pub name_map : HashMap <AttrName, AttrIndex>,
    pub logger   : Logger,
}

// === Types ===

type AttrName      = String;
type AttrIndex     = nested::Index;
type AttrBlr       = attr::Builder<f32>;
type Attr<OnDirty> = SharedAttr<f32, OnChildChange<OnDirty>>;

// === Implementation ===

impl<OnDirty: Clone> Scope<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let seq      = Seq::new(logger.clone(), on_dirty);
        let name_map = default();
        Self { seq, name_map, logger }
    }
}

impl<OnDirty: Callback0> Scope<OnDirty> {
    pub fn add_attribute<Name: Str>(&mut self, name: Name, bldr: AttrBlr) -> Attr<OnDirty> {
        let name = name.as_ref().to_string();
        let bldr = bldr.logger(self.logger.sub(&name));
        group!(self.logger, format!("Adding attribute '{}'.", name), {
            self.seq.add_and_clone(|callback| Attr::build(bldr, callback))
        })
    }

    pub fn add_instance(&mut self) {
        self.seq.children.iter_mut().for_each(|attr| attr.add_element());
        let max_size = self.seq.children.iter().fold(0, |s, t| s + t.len());
        self.logger.info("!!!");
        self.logger.info(fmt!("{}", max_size));
    }
}
