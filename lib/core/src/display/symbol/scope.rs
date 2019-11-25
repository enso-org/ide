use crate::prelude::*;

use crate::dirty;
use crate::data::function::callback::*;
use crate::display::symbol::buffer as attr;
use crate::display::symbol::buffer::item::Item;
use crate::system::web::fmt;
use crate::system::web::group;
use crate::system::web::Logger;
use crate::closure;
use crate::data::opt_vec::OptVec;
use crate::dirty::traits::*;
use eval_tt::*;
use crate::{promote, promote_all, promote_buffer_types};
use crate::display::symbol::buffer::IsBuffer;


#[derive(Derivative)]
#[derivative(Copy, Clone, Debug(bound="Ix: Debug"))]
pub struct TypedIndex<Ix, T> {
    pub ix  : Ix,
    phantom : PhantomData<T>
}

impl<Ix, T> TypedIndex<Ix, T> {
    pub fn unsafe_new(ix: Ix) -> Self {
        let phantom = PhantomData;
        Self { ix, phantom }
    }
}

// =============
// === Scope ===
// =============

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scope <OnDirty> {
    pub attributes      : OptVec<AnyBuffer<OnDirty>>,
    pub attribute_dirty : BufferDirty<OnDirty>,
    pub shape_dirty     : ShapeDirty<OnDirty>,
    pub name_map        : HashMap <BufferName, AnyBufferIndex>,
    pub logger          : Logger,
    instance_count      : usize
}

// === Types ===

pub type AnyBufferIndex           = usize;
pub type BufferIndex <T, OnDirty> = TypedIndex<usize, Buffer<T, OnDirty>>;
pub type BufferName               = String;
pub type BufferDirty <OnDirty>    = dirty::SharedBitField<u64, OnDirty>;
pub type ShapeDirty     <OnDirty>    = dirty::SharedBool<OnDirty>;

promote_buffer_types! {[BufferOnSet, BufferOnResize] attr}
#[macro_export]
macro_rules! promote_scope_types { ($callbacks:tt $module:ident) => {
    crate::promote_buffer_types! { $callbacks $module }
    promote! { $callbacks $module [Scope,BufferIndex<T>] }
};}

// === Callbacks ===

closure! {
fn attribute_on_set<C:Callback0> (dirty:BufferDirty<C>, ix: usize) ->
    BufferOnSet { || dirty.set(ix) }
}

closure! {
fn attribute_on_resize<C:Callback0> (dirty:ShapeDirty<C>) ->
    BufferOnResize { || dirty.set() }
}

// === Implementation ===

impl<OnDirty: Clone> Scope<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        logger.info("Initializing.");
        let on_dirty2       = on_dirty.clone();
        let attr_logger     = logger.sub("attr_dirty");
        let shape_logger    = logger.sub("shape_dirty");
        let attribute_dirty = BufferDirty::new(attr_logger,on_dirty2);
        let shape_dirty     = ShapeDirty::new(shape_logger, on_dirty);
        let attributes      = default();
        let name_map        = default();
        let instance_count  = 0;
        Self { attributes, attribute_dirty, shape_dirty, name_map, logger, instance_count }
    }
}

impl<OnDirty: Callback0 + 'static> Scope<OnDirty> {
    pub fn add_attribute<Name: Str, T: Item>
    ( &mut self
    , name: Name
    , bldr: attr::Builder<T>
    ) -> BufferIndex<T, OnDirty>
    where AnyBuffer<OnDirty>: From<Buffer<T, OnDirty>> {
        let ix = self._add_attribute(name, bldr);
        BufferIndex::<T, OnDirty>::unsafe_new(ix)
    }

    fn _add_attribute<Name: Str, T: Item>
    (&mut self, name: Name, bldr: attr::Builder<T>) -> AnyBufferIndex
    where AnyBuffer<OnDirty>: From<Buffer<T, OnDirty>> {
        let name        = name.as_ref().to_string();
        let bldr        = bldr.logger(self.logger.sub(&name));
        let attr_dirty  = self.attribute_dirty.clone();
        let shape_dirty = self.shape_dirty.clone();
        let ix          = self.attributes.reserve_ix();
        group!(self.logger, "Adding buffer '{}' at index {}.", name, ix, {
            let on_set    = attribute_on_set(attr_dirty, ix);
            let on_resize = attribute_on_resize(shape_dirty);
            let attr      = Buffer::build(bldr, on_set, on_resize);
            self.attributes.set(ix, AnyBuffer::from(attr));
            self.name_map.insert(name, ix);
            self.shape_dirty.set();
            ix
        })
    }

    pub fn add_instance(&mut self) -> usize {
        let ix = self.instance_count;
        self.instance_count += 1;
        self.attributes.iter_mut().for_each(|attr| attr.add_element());
        ix
    }

    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            for i in 0..self.attributes.len() {
                if self.attribute_dirty.check_for(&(i, )) {
                    self.attributes[i].update()
                }
            }
            self.attribute_dirty.unset()
        })
    }
}


impl<T, OnDirty>
Index<TypedIndex<usize, T>> for Scope<OnDirty>
where for<'t> &'t T: TryFrom<&'t AnyBuffer<OnDirty>> {
    type Output = T;
    fn index(&self, t: TypedIndex<usize, T>) -> &Self::Output {
        match self.attributes.index(t.ix).try_into() {
            Ok(t) => t,
            _     => panic!("Unmatched types for given index.")
        }
    }
}

impl<T, OnDirty>
IndexMut<TypedIndex<usize, T>> for Scope<OnDirty>
where for<'t> &'t     T: TryFrom<&'t     AnyBuffer<OnDirty>>,
      for<'t> &'t mut T: TryFrom<&'t mut AnyBuffer<OnDirty>> {
    fn index_mut(&mut self, t: TypedIndex<usize, T>) -> &mut Self::Output {
        match self.attributes.index_mut(t.ix).try_into() {
            Ok(t) => t,
            _     => panic!("Unmatched types for given index.")
        }
    }
}
