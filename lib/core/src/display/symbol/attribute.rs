use crate::prelude::*;

use crate::data::function::callback::*;
use crate::dirty;
use crate::system::web::Logger;
use crate::system::web::fmt;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

// =====================
// === ObservableVec ===
// =====================

#[derive(Derivative)]
#[derivative(Debug(bound="T:Debug"))]
struct ObservableVec<T, OnSet = NoCallback> {
    pub vec:    Vec<T>,
    pub on_set: Callback<OnSet>,
}

impl<T, OnSet> ObservableVec<T, OnSet> {
    pub fn new(on_set: OnSet) -> Self {
        Self::new_from(Default::default(), on_set)
    }

    pub fn new_from(vec: Vec<T>, on_set: OnSet) -> Self {
        let on_set = Callback(on_set);
        Self { vec, on_set }
    }
}

impl<T, OnSet, I: SliceIndex<[T]>> Index<I> for ObservableVec<T, OnSet> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.vec.index(index)
    }
}

impl<T, OnSet: Callback0> IndexMut<usize> for ObservableVec<T, OnSet> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.on_set.call();
        self.vec.index_mut(index)
    }
}

// =================
// === Attribute ===
// =================

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound="Item:Debug"))]
pub struct Attribute <Item = f32, OnDirty = NoCallback> {
    pub buffer : Buffer <Item, OnDirty>,
    pub dirty  : Dirty  <OnDirty>,
    pub logger : Logger,
}

// === Types ===

pub type DirtyType      <Ix, OnDirty>   = dirty::SharedRange<Ix, OnDirty>;
pub type Dirty          <OnDirty>       = DirtyType<usize, OnDirty>;
pub type Buffer         <Item, OnDirty> = ObservableVec<Item, OnBufferChange<OnDirty>>;
pub type OnBufferChange <OnDirty>       = impl Fn(usize);

// === Implementation ===

fn buffer_on_change<OnDirty: Callback0>(dirty: &Dirty<OnDirty>) -> OnBufferChange<OnDirty> {
    let dirty = dirty.clone();
    move |ix| dirty.set(ix)
}

impl<Item, OnDirty: Callback0> Attribute<Item, OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        Self::new_from(Default::default(), logger, on_dirty)
    }

    pub fn new_from(buffer: Vec<Item>, logger: Logger, on_dirty: OnDirty) -> Self {
        logger.info(fmt!("Creating new {} attribute.", type_name::<Item>()));
        let dirty_logger = logger.sub("dirty");
        let dirty        = Dirty::new(on_dirty, dirty_logger);
        let buffer       = ObservableVec::new_from(buffer, buffer_on_change(&dirty));
        Self { buffer, dirty, logger }
    }

    pub fn build(builder: Builder<Item>, on_dirty: OnDirty) -> Self {
        let buffer = builder._buffer.unwrap_or_else(Default::default);
        let logger = builder._logger.unwrap_or_else(Default::default);
        Self::new_from(buffer, logger, on_dirty)
    }
}

impl<Item> Attribute<Item> {
    pub fn builder() -> Builder<Item> {
        Default::default()
    }
}


// =======================
// === SharedAttribute ===
// =======================

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound="Item:Debug"))]
pub struct SharedAttribute <Item = f32, OnDirty = NoCallback> {
    pub data: Rc<RefCell<Attribute<Item, OnDirty>>>
}

impl<Item, OnDirty: Callback0> SharedAttribute<Item, OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        Self::new_from(Default::default(), logger, on_dirty)
    }

    pub fn new_from(buffer: Vec<Item>, logger: Logger, on_dirty: OnDirty) -> Self {
        let data = Rc::new(RefCell::new(Attribute::new_from(buffer, logger, on_dirty)));
        Self { data }
    }

    pub fn build(builder: Builder<Item>, on_dirty: OnDirty) -> Self {
        let data = Rc::new(RefCell::new(Attribute::build(builder, on_dirty)));
        Self { data }
    }
}

impl<Item, OnDirty> SharedAttribute<Item, OnDirty> {
    pub fn new_ref(&self) -> Self {
        Self { data: Rc::clone(&self.data) }
    }
}

// ===============
// === Builder ===
// ===============

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Builder<Item> {
    pub _buffer : Option <Vec <Item>>,
    pub _logger : Option <Logger>
}

impl<Item> Builder<Item> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn buffer(self, val: Vec <Item>) -> Self {
        Self { _buffer: Some(val), _logger: self._logger }
    }

    pub fn logger(self, val: Logger) -> Self {
        Self { _buffer: self._buffer, _logger: Some(val) }
    }
}