use crate::prelude::*;

use crate::data::function::callback::*;
use crate::display::symbol::attribute as attr;
use crate::display::symbol::attribute::IsAttribute;
use crate::display::symbol::attribute::Shape;
use crate::display::symbol::attribute::SharedAttribute;
use crate::display::symbol::nested;
use crate::display::symbol::nested::OnChildChange;
use crate::display::symbol::nested::Seq;
use crate::system::web::fmt;
use crate::system::web::group;
use crate::system::web::Logger;

// =============
// === Scope ===
// =============

// === Types ===

type AttributeName         = String;
type AttributeIndex        = nested::Index;
type Attribute<T, OnDirty> = attr::SharedAttribute<T, OnChildChange<OnDirty>>;
type AnyAttribute<OnDirty> = attr::AnyAttribute<OnChildChange<OnDirty>>;


// === Definition ===

// #[derive(Shrinkwrap)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scope <OnDirty = NoCallback> {
    pub seq      : Seq <AnyAttribute<OnDirty>, OnDirty>,
    pub name_map : HashMap <AttributeName, AttributeIndex>,
    pub logger   : Logger,
}

// === Implementation ===

impl<OnDirty: Clone> Scope<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let seq      = Seq::new(logger.clone(), on_dirty);
        let name_map = default();
        Self { seq, name_map, logger }
    }
}

impl<OnDirty: Callback0 + 'static> Scope<OnDirty> {
    pub fn add_attribute<Name: Str, T: Shape>
            ( &mut self
            , name: Name
            , bldr: attr::Builder<T>
            ) -> Attribute<T, OnDirty>
            where AnyAttribute<OnDirty>: From<Attribute<T, OnDirty>> {
        let name = name.as_ref().to_string();
        let bldr = bldr.logger(self.logger.sub(&name));
        group!(self.logger, format!("Adding attribute '{}'.", name), {
            self.seq.add_and_clone(|callback| {
                let out = Attribute::build(bldr, callback);
                let out2 = out.clone();
                (AnyAttribute::from(out2), out)
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

