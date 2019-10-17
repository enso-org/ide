use crate::prelude::*;

use crate::data::function::callback::*;
use crate::dirty;
use crate::system::web::Logger;
use crate::system::web::fmt;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

use nalgebra::dimension::Dim;
use nalgebra::dimension::U1;
use nalgebra::dimension::DimName;


pub trait Shape {
    type Item;
    type Dim: DimName;

    fn item_count() -> usize {
        <Self::Dim as DimName>::dim()
    }
}

impl Shape for f32 {
    type Item = Self;
    type Dim  = nalgebra::dimension::U1;
}

// fn foo<I,T: Shape<Item=I>>() -> <T as Shape>::Item 
//     where <T as Shape>::Item : Default {
//     Default::default()
// }

// =====================
// === ObservableVec ===
// =====================

#[derive(Derivative)]
#[derivative(Debug(bound="T:Debug"))]
pub struct ObservableVec<T, OnSet = NoCallback> {
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
#[derivative(Debug(bound="<T as Shape>::Item:Debug"))]
pub struct Attr<T: Shape = f32, OnItemMod = NoCallback> {
    pub buffer : Buffer <T, OnItemMod>,
    pub dirty  : Dirty  <OnItemMod>,
    pub logger : Logger,
}

// === Types ===

pub type DirtyType      <Ix, OnDirty> = dirty::SharedRange<Ix, OnDirty>;
pub type Dirty          <OnDirty>     = DirtyType<usize, OnDirty>;
pub type Buffer         <T, OnDirty>  = ObservableVec<<T as Shape>::Item, OnBufferChange<OnDirty>>;
pub type RawBuffer      <T>           = Vec<<T as Shape>::Item>;
pub type OnBufferChange <OnDirty>     = impl Fn(usize);

// === Implementation ===

fn buffer_on_change<OnDirty: Callback0>(dirty: &Dirty<OnDirty>) -> OnBufferChange<OnDirty> {
    let dirty = dirty.clone();
    move |ix| dirty.set(ix)
}

impl<T: Shape, OnDirty: Callback0> Attr<T, OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        Self::new_from(Default::default(), logger, on_dirty)
    }

    pub fn new_from(buffer: RawBuffer<T>, logger: Logger, on_dirty: OnDirty) -> Self {
        logger.info(fmt!("Creating new {} attribute.", type_name::<T>()));
        let dirty_logger = logger.sub("dirty");
        let dirty        = Dirty::new(on_dirty, dirty_logger);
        let buffer       = ObservableVec::new_from(buffer, buffer_on_change(&dirty));
        Self { buffer, dirty, logger }
    }

    pub fn build(builder: Builder<T>, on_dirty: OnDirty) -> Self {
        let buffer = builder._buffer.unwrap_or_else(Default::default);
        let logger = builder._logger.unwrap_or_else(Default::default);
        Self::new_from(buffer, logger, on_dirty)
    }
}

impl<T: Shape, OnDirty> Attr<T, OnDirty> {
    pub fn len(&self) -> usize {
        self.buffer.vec.len()
    }
}

impl<T: Shape<Item=I>, I: Default + Clone, OnDirty> Attr<T, OnDirty> {
    pub fn add_element(&mut self) {
        self.add_elements(1);
    }

    pub fn add_elements(&mut self, elem_count: usize) {
        let item_count = elem_count * <T as Shape>::item_count();
        self.buffer.vec.extend(iter::repeat(default()).take(item_count));
    }
}

impl<T: Shape> Attr<T> {
    pub fn builder() -> Builder<T> {
        Default::default()
    }
}


// =======================
// === SharedAttribute ===
// =======================

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound="<T as Shape>::Item:Debug"))]
#[derivative(Clone(bound=""))]
pub struct SharedAttr<T: Shape = f32, OnDirty = NoCallback> {
    pub data: Rc<RefCell<Attr<T, OnDirty>>>
}

impl<T: Shape, OnDirty: Callback0> SharedAttr<T, OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        Self::new_from(Default::default(), logger, on_dirty)
    }

    pub fn new_from(buffer: RawBuffer<T>, logger: Logger, on_dirty: OnDirty) -> Self {
        let data = Rc::new(RefCell::new(Attr::new_from(buffer, logger, on_dirty)));
        Self { data }
    }

    pub fn build(builder: Builder<T>, on_dirty: OnDirty) -> Self {
        let data = Rc::new(RefCell::new(Attr::build(builder, on_dirty)));
        Self { data }
    }
}

impl<T: Shape, OnDirty> SharedAttr<T, OnDirty> {
    pub fn new_ref(&self) -> Self {
        Self { data: Rc::clone(&self.data) }
    }

    pub fn len(&self) -> usize {
        self.data.borrow().len()
    }
}

impl<T: Shape<Item=I>, I: Default + Clone, OnDirty> SharedAttr<T, OnDirty> {
    pub fn add_element(&self) {
        self.data.borrow_mut().add_element()
    }
}

// ===============
// === Builder ===
// ===============

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Builder<T: Shape> {
    pub _buffer : Option <RawBuffer <T>>,
    pub _logger : Option <Logger>
}

impl<T: Shape> Builder<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn buffer(self, val: RawBuffer <T>) -> Self {
        Self { _buffer: Some(val), _logger: self._logger }
    }

    pub fn logger(self, val: Logger) -> Self {
        Self { _buffer: self._buffer, _logger: Some(val) }
    }
}