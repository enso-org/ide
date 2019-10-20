use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::shared::Shared;
use crate::dirty;
use crate::system::web::Logger;
use crate::system::web::fmt;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

use nalgebra;
use nalgebra::dimension::Dim;
use nalgebra::dimension::{U1, U2, U3};
use nalgebra::dimension::DimName;
use nalgebra::Scalar;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use nalgebra::Matrix;
use nalgebra::MatrixMN;



pub trait TypeDebugName {
    fn type_debug_name() -> String; 
}

impl<T> TypeDebugName for T {
    default fn type_debug_name() -> String {
        type_name::<Self>().to_string()
    }
}




// =============
// === Shape ===
// =============

// === Definition === 

pub trait Shape: TypeDebugName {
    type Item;
    type Dim: DimName;

    fn item_count() -> usize {
        <Self::Dim as DimName>::dim()
    }

    fn from_buffer(buffer: &[Self::Item]) -> &[Self] 
        where Self: std::marker::Sized;

    fn from_buffer_mut(buffer: &mut [Self::Item]) -> &mut [Self] 
        where Self: std::marker::Sized;
}


// === Instances === 

impl Shape for f32 {
    type Item = Self;
    type Dim  = U1;

    fn from_buffer     (buffer: &    [Self::Item]) -> &    [Self] { buffer }
    fn from_buffer_mut (buffer: &mut [Self::Item]) -> &mut [Self] { buffer }
}

impl<T: Scalar, R: DimName, C: DimName> Shape for MatrixMN<T, R, C> where 
        nalgebra::DefaultAllocator : nalgebra::allocator::Allocator<T, R, C> {
    type Item = T;
    type Dim  = R;

    fn from_buffer(buffer: &[Self::Item]) -> &[Self] {
        unsafe {
            let len = buffer.len() / Self::item_count();
            std::slice::from_raw_parts(buffer.as_ptr().cast(), len)
        } 
    }

    fn from_buffer_mut(buffer: &mut [Self::Item]) -> &mut [Self] {
        unsafe {
            let len = buffer.len() / Self::item_count();
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast(), len)
        } 
    }
}

impl <T: Scalar, R: DimName, C: DimName> TypeDebugName for MatrixMN<T, R, C> where 
        nalgebra::DefaultAllocator : nalgebra::allocator::Allocator<T, R, C> {
    fn type_debug_name() -> String {
        let col  = <C as DimName>::dim();
        let row  = <R as DimName>::dim();
        let item = type_name::<T>();
        match col {
            1 => format!("Vector{}<{}>", row, item),
            _ => format!("Matrix{}x{}<{}>", row, col, item)
        }
    }
}

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
pub struct Attribute<T: Shape, OnItemMod = NoCallback> {
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

impl<T: Shape, OnDirty: Callback0> Attribute<T, OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        Self::new_from(Default::default(), logger, on_dirty)
    }

    pub fn new_from(buffer: RawBuffer<T>, logger: Logger, on_dirty: OnDirty) -> Self {
        logger.info(fmt!("Creating new {} attribute.", T::type_debug_name()));
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

impl<T: Shape, OnDirty> Attribute<T, OnDirty> {
    pub fn len(&self) -> usize {
        self.buffer.vec.len()
    }
}

pub trait AddElementCtx = Shape where <Self as Shape>::Item: Default + Clone;
impl<T: AddElementCtx, OnDirty> Attribute<T, OnDirty> {
    pub fn add_element(&mut self) {
        self.add_elements(1);
    }

    pub fn add_elements(&mut self, elem_count: usize) {
        let item_count = elem_count * <T as Shape>::item_count();
        self.buffer.vec.extend(iter::repeat(default()).take(item_count));
    }
}

impl<T: Shape> Attribute<T> {
    pub fn builder() -> Builder<T> {
        Default::default()
    }
}

impl<T: Shape, OnSet, I: SliceIndex<[T]>> Index<I> for Attribute<T, OnSet> {
    type Output = I::Output;
    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &T::from_buffer(&self.buffer.vec)[index]
    }
}

impl<T: Shape, OnSet, I: SliceIndex<[T]>> IndexMut<I> for Attribute<T, OnSet> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut T::from_buffer_mut(&mut self.buffer.vec)[index]
    }
}




// =======================
// === SharedAttribute ===
// =======================

// === Definition ===

#[derive(Shrinkwrap)]
#[derive(Derivative)]
#[derivative(Debug(bound="<T as Shape>::Item:Debug"))]
pub struct SharedAttribute<T: Shape, OnDirty = NoCallback> {
    pub data: Shared<Attribute<T, OnDirty>>
}

impl<T: Shape, OnDirty: Callback0> SharedAttribute<T, OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        Self::new_from(Default::default(), logger, on_dirty)
    }

    pub fn new_from(buffer: RawBuffer<T>, logger: Logger, on_dirty: OnDirty) -> Self {
        let data = Shared::new(Attribute::new_from(buffer, logger, on_dirty));
        Self { data }
    }

    pub fn build(builder: Builder<T>, on_dirty: OnDirty) -> Self {
        let data = Shared::new(Attribute::build(builder, on_dirty));
        Self { data }
    }
}

impl<T: Shape, OnDirty> SharedAttribute<T, OnDirty> {
    pub fn clone_ref(&self) -> Self {
        Self { data: self.data.clone_ref() }
    }

    pub fn len(&self) -> usize {
        self.data.borrow().len()
    }
}

impl<T: AddElementCtx, OnDirty> SharedAttribute<T, OnDirty> {
    pub fn add_element(&self) {
        self.data.borrow_mut().add_element()
    }
}

impl<T: Shape, OnDirty, I: SliceIndex<[T]>> Index<I> for SharedAttribute<T, OnDirty> {
    type Output = I::Output;
    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}

// impl<T: Shape, OnDirty> Deref for SharedAttribute<T, OnDirty> {
//     type Target = Ref<Attribute<T, OnDirty>>;

//     fn deref(&self) -> &Self::Target {
//         &self.data.borrow()
//     }
// }



// struct FooGuard<'t, T, OnDirty> {
//     guard: Ref<'t, Attribute<T, OnDirty>>,
// }

// impl<'t, T, OnDirty> Deref for FooGuard<'t, T, OnDirty> {
//     type Target = Vec<i32>;

//     fn deref(&self) -> &Self::Target {
//         &self.guard
//     }
// }

// impl Foo {
//     pub fn get_items(&self) -> FooGuard {
//         FooGuard {
//             guard: self.interior.borrow(),
//         }
//     }
// }

// impl<T: Shape, OnSet, I: SliceIndex<[T]>> Index<I> 
//         for SharedAttribute<T, OnSet> {
//     type Output = I::Output;
//     #[inline]
//     fn index(&self, index: I) -> &Self::Output {
//         &self.data.borrow()[index]
//     }
// }

// ==========================
// === AnySharedAttributeibute ===
// ==========================

use enum_dispatch::*;


macro_rules! cartesian_impl {
    ($out:tt [] $b:tt $init_b:tt, $f:ident) => {
        $f!{ $out }
    };
    ($out:tt [$a:ident, $($at:tt)*] [] $init_b:tt, $f:ident) => {
        cartesian_impl!{ $out [$($at)*] $init_b $init_b, $f }
    };
    ([$($out:tt)*] [$a:ident, $($at:tt)*] [$b:ident, $($bt:tt)*] $init_b:tt 
    ,$f:ident) => {
        cartesian_impl!{ 
            [$($out)* ($a, $b),] [$a, $($at)*] [$($bt)*] $init_b, $f 
        }
    };
}

macro_rules! cartesian {
    ([$($a:tt)*], [$($b:tt)*], $f:ident) => {
        cartesian_impl!{ [] [$($a)*,] [$($b)*,] [$($b)*,], $f }
    };
}

macro_rules! mk_any_shape_impl {
    ([$(($base:ident, $param:ident)),*,]) => {
        paste::item! {
            #[enum_dispatch(IsAttribute)]
            #[derive(Derivative)]
            #[derivative(Debug(bound=""))]
            pub enum AnyAttribute<OnDirty> {
                $([<Variant $base For $param>](SharedAttribute<$base<$param>, OnDirty>),)*
            } 
        }
    }
}

macro_rules! mk_any_shape {
    ($bases:tt, $params:tt) => {
        cartesian!($bases, $params, mk_any_shape_impl);
    }
}

type Identity<T> = T;
mk_any_shape!([Identity, Vector2, Vector3, Vector4], [f32]);



#[enum_dispatch]
pub trait IsAttribute<OnDirty> {
    fn add_element(&self);
    fn len(&self) -> usize;
}


// impl IsShape for Vector2<f32>{}
// impl IsShape for Vector3<f32>{}







// // mk_any_shape!([(Vector2,f32),(Vector3,f32),]);

// pub trait IsAttribute<OnDirty> {
//     fn add_element(&self);
//     fn len(&self) -> usize;
// }

// pub struct AnyAttribute<OnDirty> (pub Box<dyn IsAttribute<OnDirty>>);

// pub trait IsAttributeCtx = AddElementCtx;
// impl<T: IsAttributeCtx, OnDirty> IsAttribute<OnDirty> for SharedAttribute<T, OnDirty> {
//     fn add_element(&self) {
//         self.add_element()
//     }
//     fn len(&self) -> usize {
//         self.len()
//     }
// }

// impl<T> std::fmt::Debug for AnyAttribute<T> {
//     fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         write!(fmt, "AnyAttribute")
//     }
// }

// impl<OnDirty> AnyAttribute<OnDirty> {
//     pub fn add_element(&self) {
//         self.0.add_element()
//     }
//     pub fn len(&self) -> usize {
//         self.0.len()
//     }
// }

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