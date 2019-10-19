use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::dirty;
use crate::display::symbol::attr;
use crate::display::symbol::attr::Shape;
use crate::display::symbol::attr::SharedAttr;
use crate::display::symbol::attr::AnyAttribute;
use crate::system::web::Logger;
use crate::system::web::group;
use crate::system::web::fmt;
use std::slice::SliceIndex;
use crate::display::symbol::nested::Seq;
use crate::display::symbol::nested::OnChildChange;
use crate::display::symbol::nested;
use crate::display::symbol::attr::IsAttribute;

// =============
// === Scope ===
// =============

// === Definition ===

// #[derive(Shrinkwrap)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scope <OnDirty = NoCallback> {
    pub seq      : Seq <AnyAttribute<OnChildChange<OnDirty>>, OnDirty>,
    pub name_map : HashMap <AttrName, AttrIndex>,
    pub logger   : Logger,
}

// === Types ===

type AttrName      = String;
type AttrIndex     = nested::Index;
type AttrBlr<T>    = attr::Builder<T>;
type Attr<T, OnDirty> = SharedAttr<T, OnChildChange<OnDirty>>;
type AnyAttr<OnDirty> = AnyAttribute<OnChildChange<OnDirty>>;

// === Implementation ===

impl<OnDirty: Clone> Scope<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let seq      = Seq::new(logger.clone(), on_dirty);
        let name_map = default();
        Self { seq, name_map, logger }
    }
}

impl<OnDirty: Callback0+ 'static> Scope<OnDirty> {
    pub fn add_attribute<Name: Str, T: Shape>(&mut self, name: Name, bldr: AttrBlr<T>) -> Attr<T, OnDirty> where 
    AnyAttr<OnDirty>: From<Attr<T, OnDirty>>{
        let name = name.as_ref().to_string();
        let bldr = bldr.logger(self.logger.sub(&name));
        group!(self.logger, format!("Adding attribute '{}'.", name), {
            self.seq.add_and_clone(|callback| {
                let out = Attr::build(bldr, callback);
                let out2 = out.clone();
                (AnyAttribute::from(out2),out)
            })
        })
    }

    pub fn add_instance(&mut self) {
        self.seq.children.iter_mut().for_each(|attr| attr.add_element());
        let max_size = self.seq.children.iter().fold(0, |s, t| s + t.len());
        self.logger.info("!!!");
        self.logger.info(fmt!("{}", max_size));
    }
}
