use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::shared::Shared;
use crate::dirty;
use crate::system::web::Logger;
use crate::system::web::fmt;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;
use crate::tp::debug::TypeDebugName;
use std::iter::Extend;

use nalgebra;
use nalgebra::dimension::{U1, U2, U3};
use nalgebra::dimension::DimName;
use nalgebra::Scalar;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use nalgebra::Matrix;
use nalgebra::MatrixMN;

use crate::closure;



macro_rules! type_family { 
    ($name:ident) => {
        paste::item! {
            trait [<Has $name>] { type $name; }
            type $name<T> = <T as [<Has $name>]>::$name;
        }
    };
}

macro_rules! type_instance { 
    ($name:ident<$param:ident> = $expr:expr) => {
        paste::item! {
            impl [<Has $name>] for $param { 
                type $name = $expr; 
            }
        }
    };
}


// type_family!(OnSet);
// type_family!(OnResize);

// type_instance!(OnSet<i32> = String);


// #[derive(Shrinkwrap)]
// #[shrinkwrap(mutable)]
// struct Buf<T: Shape> {
//     raw_data: Vec<T::Item>
// }




// impl<T: Shape> AsRef<[Item<T>]> for Buf<T> {
//     // This is safe, as we are casting between item and container, and we 
//     // update the slice length accordingly. The container knows it's item 
//     // count, like `Vector2<Item>`.
//     fn as_ref(&self) -> &[Item<T>] {
//         unsafe {
//             let len = self.len() / T::item_count();
//             std::slice::from_raw_parts(self.as_ptr().cast(), len)
//         } 
//     }
// }

// impl<T: Shape> AsMut<[Item<T>]> for Buf<T> {
//     // This is safe, as we are casting between item and container, and we 
//     // update the slice length accordingly. The container knows it's item 
//     // count, like `Vector2<Item>`.
//     fn as_mut(&mut self) -> &mut [Item<T>] {
//         unsafe {
//             let len = self.len() / T::item_count();
//             std::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), len)
//         } 
//     }
// }

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

    fn empty() -> Self;

    fn from_buffer(buffer: &[Self::Item]) -> &[Self] 
        where Self: std::marker::Sized;

    fn from_buffer_mut(buffer: &mut [Self::Item]) -> &mut [Self] 
        where Self: std::marker::Sized;
}

// === Type Families ===

type Item <T> = <T as Shape>::Item;
type Dim  <T> = <T as Shape>::Dim;

// === Shapes for numbers === 

impl Shape for i32 {
    type Item = Self;
    type Dim  = U1;

    fn empty           ()                          -> Self        { 0 }
    fn from_buffer     (buffer: &    [Self::Item]) -> &    [Self] { buffer }
    fn from_buffer_mut (buffer: &mut [Self::Item]) -> &mut [Self] { buffer }
}

impl Shape for f32 {
    type Item = Self;
    type Dim  = U1;

    fn empty           ()                          -> Self        { 0.0 }
    fn from_buffer     (buffer: &    [Self::Item]) -> &    [Self] { buffer }
    fn from_buffer_mut (buffer: &mut [Self::Item]) -> &mut [Self] { buffer }
}

// === Shapes for matrixes === 

pub trait AllocatorCtx<T: Scalar, R: DimName, C: DimName> = where 
    nalgebra::DefaultAllocator : nalgebra::allocator::Allocator<T, R, C>;

impl<T: Scalar + Default, R: DimName, C: DimName> 
Shape for MatrixMN<T, R, C> where Self: AllocatorCtx<T, R, C> {
    type Item = T;
    type Dim  = R;

    fn empty() -> Self {
        Self::repeat(default())
    }

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

impl <T: Scalar, R: DimName, C: DimName> 
TypeDebugName for MatrixMN<T, R, C> where Self: AllocatorCtx<T, R, C> {
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

// ==================
// === Observable ===
// ==================

#[derive(Shrinkwrap)]
#[derive(Derivative)]
#[derivative(Debug(bound="T:Debug"))]
pub struct Observable<T, OnSet, OnResize> {
    #[shrinkwrap(main_field)]
    pub data      : T,
    pub on_set    : Callback<OnSet>,
    pub on_resize : Callback<OnResize>,
}

impl<T: Default, OnSet, OnResize>
Observable<T, OnSet, OnResize> {
    pub fn new(on_set: OnSet, on_resize: OnResize) -> Self {
        Self::new_from(default(), on_set, on_resize)
    }

    pub fn new_from(data: T, on_set: OnSet, on_resize: OnResize) -> Self {
        let on_set    = Callback(on_set);
        let on_resize = Callback(on_resize);
        Self { data, on_set, on_resize }
    }
}

impl<T: Index<I>, OnSet, OnResize, I> 
Index<I> for Observable<T, OnSet, OnResize> {
    type Output = <T as Index<I>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: IndexMut<I>, OnSet: Callback1<I>, OnResize, I: Copy> 
IndexMut<I> for Observable<T, OnSet, OnResize> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.on_set.call(index);
        &mut self.data[index]
    }
}

impl <T: Extend<S>, S, OnSet, OnResize: Callback0> 
Extend<S> for Observable<T, OnSet, OnResize> {
    #[inline]
    fn extend<I: IntoIterator<Item = S>>(&mut self, iter: I) {
        self.on_resize.call();
        self.data.extend(iter)
    }
}


//////////////////////////////////////////////////////



// =================
// === Attribute ===
// =================

// === Definition ===

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derive(Derivative)]
#[derivative(Debug(bound="T:Debug"))]
pub struct Attribute<T: Shape, OnSet, OnResize> {
    #[shrinkwrap(main_field)]
    pub buffer       : Buffer      <T, OnSet, OnResize>,
    pub set_dirty    : SetDirty    <OnSet>,
    pub resize_dirty : ResizeDirty <OnResize>,
    pub logger       : Logger,
}

// === Types ===

pub trait SetDirtyCtx    <Callback> = dirty::RangeCtx<Callback>;
pub trait ResizeDirtyCtx <Callback> = dirty::BoolCtx<Callback>;
pub type  SetDirty       <Callback> = dirty::SharedRange<usize, Callback>;
pub type  ResizeDirty    <Callback> = dirty::SharedBool<Callback>;

// pub type  Buffer <T, OnSet, OnResize> = Observable
//     < Vec<T>
//     , Closure_buffer_on_set_handler    <OnSet>
//     , Closure_buffer_on_resize_handler <OnResize>
//     >;



pub type  Buffer <T, OnSet, OnResize> = Observable
    < Vec<T>
    , Closure_buffer_on_set_handler<OnSet>
    , Closure_buffer_on_resize_handler<OnResize>
    >;

// === Callbacks ===

closure!(buffer_on_resize_handler<Callback: Callback0>
    (dirty: ResizeDirty<Callback>) || { dirty.set() });

closure!(buffer_on_set_handler<Callback: Callback0>
    (dirty: SetDirty<Callback>) |ix: usize| { dirty.set(ix) });

// pub type Closure_buffer_on_set_handler<Callback> = impl Fn(usize) + Clone;
// pub fn buffer_on_set_handler<Callback: Callback0>
// (dirty: SetDirty<Callback>) -> Closure_buffer_on_set_handler<Callback> {
//     move |ix| { dirty.set(ix) }
// }

// pub type Closure_buffer_on_set_handler<Callback> = impl Fn(usize) + Clone;

// === Instances ===

impl<T: Shape, OnSet: Callback0 + 'static, OnResize: Callback0 + 'static> 
Attribute<T, OnSet, OnResize> {
    pub fn new_from
    (vec: Vec<T>, logger: Logger, on_set: OnSet, on_resize: OnResize) -> Self {
        logger.info(fmt!("Creating new {} attribute.", T::type_debug_name()));
        let set_logger     = logger.sub("set_dirty");
        let resize_logger  = logger.sub("resize_dirty");
        let set_dirty      = SetDirty::new(on_set, set_logger);
        let resize_dirty   = ResizeDirty::new(on_resize, resize_logger);
        let buff_on_resize = buffer_on_resize_handler(resize_dirty.clone());
        let buff_on_set    = buffer_on_set_handler(set_dirty.clone());
        let buffer         = Buffer::new_from(vec, buff_on_set, buff_on_resize);
        Self { buffer, set_dirty, resize_dirty, logger }
    }

    pub fn new(logger: Logger, on_set: OnSet, on_resize: OnResize) -> Self {
        Self::new_from(default(), logger, on_set, on_resize)
    }

    pub fn build(bldr: Builder<T>, on_set: OnSet, on_resize: OnResize) -> Self {
        let buffer = bldr._buffer.unwrap_or_else(default);
        let logger = bldr._logger.unwrap_or_else(default);
        Self::new_from(buffer, logger, on_set, on_resize)
    }

    pub fn builder() -> Builder<T> {
        default()
    }
}

impl<T: Shape, OnSet, OnResize> 
Attribute<T, OnSet, OnResize> {
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

pub trait AddElementCtx = Shape + Clone;
impl<T: AddElementCtx, OnSet, OnResize> 
Attribute<T, OnSet, OnResize> {
    pub fn add_element(&mut self) {
        self.add_elements(1);
    }

    pub fn add_elements(&mut self, elem_count: usize) {
        unimplemented!()
        // self.extend(iter::repeat(T::empty()).take(elem_count));
    }
}



// // =======================
// // === SharedAttribute ===
// // =======================

// // === Definition ===

// #[derive(Shrinkwrap)]
// #[derive(Derivative)]
// #[derivative(Debug(bound="T:Debug"))]
// pub struct SharedAttribute<T: Shape, OnSet, OnResize> {
//     pub data: Shared<Attribute<T, OnSet, OnResize>>
// }

// impl<T: Shape, OnSet: Callback0, OnResize: Callback0> SharedAttribute<T, OnSet, OnResize> {
//     pub fn new(logger: Logger, on_set: OnSet, on_resize: OnResize) -> Self {
//         Self::new_from(default(), logger, on_set, on_resize)
//     }

//     pub fn new_from(buffer: Vec<T>, logger: Logger, on_set: OnSet, on_resize: OnResize) -> Self {
//         let data = Shared::new(Attribute::new_from(buffer, logger, on_set, on_resize));
//         Self { data }
//     }

//     pub fn build(builder: Builder<T>, on_set: OnSet, on_resize: OnResize) -> Self {
//         let data = Shared::new(Attribute::build(builder, on_set, on_resize));
//         Self { data }
//     }

//     pub fn builder() -> Builder<T> {
//         default()
//     }
// }

// impl<T: Shape, OnSet, OnResize> SharedAttribute<T, OnSet, OnResize> {
//     pub fn clone_ref(&self) -> Self {
//         Self { data: self.data.clone_ref() }
//     }

//     pub fn len(&self) -> usize {
//         self.data.borrow().len()
//     }
// }

// impl<T: AddElementCtx, OnSet, OnResize> SharedAttribute<T, OnSet, OnResize> {
//     pub fn add_element(&self) {
//         self.data.borrow_mut().add_element()
//     }
// }

// impl<T: Shape, OnSet, OnResize, I: SliceIndex<[T]>> Index<I> for SharedAttribute<T, OnSet, OnResize> {
//     type Output = I::Output;
//     #[inline]
//     fn index(&self, index: I) -> &Self::Output {
//         &self.data[index]
//     }
// }

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

#[derive(Debug)]
pub struct BadVariant;


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
    ([$(($base:ident, $param:ident)),*,]) => { paste::item! {
        #[enum_dispatch(IsAttribute)]
        #[derive(Derivative)]
        #[derivative(Debug(bound=""))]
        pub enum AnyAttribute<OnSet, OnResize> {
            $(  [<Variant $base For $param>]
                    (Attribute<$base<$param>, OnSet, OnResize>),
            )*
        } 

        $( /////////////////////////////////////////////////////////////////////

        impl<'t, T, S> 
        TryFrom<&'t AnyAttribute<T, S>> 
        for &'t Attribute<$base<$param>, T, S> {
            type Error = BadVariant;
            fn try_from(v: &'t AnyAttribute<T, S>) 
            -> Result <&'t Attribute<$base<$param>, T, S>, Self::Error> { 
                match v {
                    AnyAttribute::[<Variant $base For $param>](a) => Ok(a),
                    _ => Err(BadVariant)
                }
            }
        }
        
        impl<'t, T, S> 
        TryFrom<&'t mut AnyAttribute<T, S>> 
        for &'t mut Attribute<$base<$param>, T, S> {
            type Error = BadVariant;
            fn try_from(v: &'t mut AnyAttribute<T, S>) 
            -> Result <&'t mut Attribute<$base<$param>, T, S>, Self::Error> { 
                match v {
                    AnyAttribute::[<Variant $base For $param>](a) => Ok(a),
                    _ => Err(BadVariant)
                }
            }
        }

        )* /////////////////////////////////////////////////////////////////////
    }
}}

macro_rules! mk_any_shape {
    ($bases:tt, $params:tt) => {
        cartesian!($bases, $params, mk_any_shape_impl);
    }
}

type Identity<T> = T;
mk_any_shape!([Identity, Vector2, Vector3, Vector4], [f32, i32]);


#[enum_dispatch]
pub trait IsAttribute<OnSet, OnResize> {
    fn add_element(&mut self);
    fn len(&self) -> usize;
}








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
    pub _buffer : Option <Vec <T>>,
    pub _logger : Option <Logger>
}

impl<T: Shape> Builder<T> {
    pub fn new() -> Self {
        default()
    }

    pub fn buffer(self, val: Vec <T>) -> Self {
        Self { _buffer: Some(val), _logger: self._logger }
    }

    pub fn logger(self, val: Logger) -> Self {
        Self { _buffer: self._buffer, _logger: Some(val) }
    }
}