use crate::prelude::*;

use crate::dirty;
use crate::data::function::callback::*;
use crate::display::symbol::attribute as attr;
use crate::display::symbol::attribute::IsAttribute;
use crate::display::symbol::attribute::Shape;
use crate::display::symbol::attribute::SharedAttribute;
use crate::system::web::fmt;
use crate::system::web::group;
use crate::system::web::Logger;
use crate::closure;

// =============
// === Scope ===
// =============

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scope <OnDirty> {
    pub attributes      : Vec<AnyAttribute<OnDirty>>,
    pub attribute_dirty : AttributeDirty<OnDirty>,
    pub shape_dirty     : ShapeDirty<OnDirty>,
    pub name_map        : HashMap <AttributeName, AttributeIndex>,
    pub logger          : Logger,
}

// === Types ===

pub type AttributeName            = String;
pub type AttributeIndex           = usize;
pub type AttributeDirty <OnDirty> = dirty::SharedBitField<u64, OnDirty>;
pub type ShapeDirty     <OnDirty> = dirty::SharedBool<OnDirty>;

pub type Attribute<T, OnDirty> = attr::SharedAttribute
    < T
    , Closure_attribute_on_set_handler<OnDirty>
    , Closure_attribute_on_resize_handler<OnDirty>
    >;

pub type AnyAttribute<OnDirty> = attr::AnyAttribute
    < Closure_attribute_on_set_handler<OnDirty>
    , Closure_attribute_on_resize_handler<OnDirty>
    >;

// === Callbacks ===

closure!(attribute_on_set_handler<Callback: Callback0>
    (dirty: AttributeDirty<Callback>, ix: AttributeIndex) || { dirty.set(ix) });

closure!(attribute_on_resize_handler<Callback: Callback0>
    (dirty: ShapeDirty<Callback>) || { dirty.set() });


// === Implementation ===

impl<OnDirty: Clone> Scope<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        logger.info("Initializing.");
        let on_dirty2       = on_dirty.clone();
        let attr_logger     = logger.sub("attr_dirty");
        let shape_logger    = logger.sub("shape_dirty");
        let attribute_dirty = AttributeDirty::new(on_dirty2, attr_logger);
        let shape_dirty     = ShapeDirty::new(on_dirty, shape_logger);
        let attributes      = Vec::new();
        let name_map        = default();
        Self { attributes, attribute_dirty, shape_dirty, name_map, logger }
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
        let ix   = self.attributes.len();
        let msg  = || format!("Adding attribute '{}' at index {}.", name, ix);
        group!(self.logger, msg(), {
            let attr_dirty  = self.attribute_dirty.clone();
            let shape_dirty = self.shape_dirty.clone();
            let on_set      = attribute_on_set_handler(attr_dirty, ix);
            let on_resize   = attribute_on_resize_handler(shape_dirty);
            let attr        = Attribute::build(bldr, on_set, on_resize);
            let any_attr    = AnyAttribute::from(attr.clone_ref());
            self.attributes.push(any_attr);
            self.name_map.insert(name, ix);
            self.shape_dirty.set();
            attr
        })
    }

    pub fn add_instance(&mut self) {
        self.attributes.iter_mut().for_each(|attr| attr.add_element());
        let max_size = self.attributes.iter().fold(0, |s, t| s + t.len());
        // self.logger.info("!!!");
        // self.logger.info(fmt!("{}", max_size));
    }
}

