//! This module re-exports a lot of useful stuff. It is not meant to be used
//! by libraries, but it is definitely usefull for bigger projects. It also
//! defines several aliases and utils which may find their place in new
//! libraries in the future.

#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![feature(specialization)]
#![feature(trait_alias)]

pub mod macros;
pub mod option;
pub mod phantom;
pub mod std_reexports;
pub mod reference;
pub mod string;
pub mod tp;
pub mod wrapper;

pub use macros::*;
pub use option::*;
pub use phantom::*;
pub use std_reexports::*;
pub use reference::*;
pub use string::*;
pub use tp::*;
pub use wrapper::*;

pub use boolinator::Boolinator;

pub use derivative::Derivative;
pub use derive_more::*;
pub use enclose::enclose;
pub use failure::Fail;
pub use ifmt::*;
pub use itertools::Itertools;
pub use lazy_static::lazy_static;
pub use num::Num;
pub use paste;
pub use shrinkwraprs::Shrinkwrap;
pub use smallvec::SmallVec;



// ================
// === CloneRef ===
// ================

/// Like `Clone` but should be implemented only for cheap reference-based clones. Using `clone_ref`
/// instead of `clone` makes the code more clear and makes it easier to predict its performance.
pub trait CloneRef: Sized + Clone {
    fn clone_ref(&self) -> Self {
        self.clone()
    }
}

impl CloneRef for () {
    fn clone_ref(&self) -> Self {}
}

impl<T:?Sized> CloneRef for Rc<T> {
    fn clone_ref(&self) -> Self {
        self.clone()
    }
}



// ===================
// === WithPhantom ===
// ===================

/// A wrapper adding a phantom type to a structure.
#[derive(Derivative)]
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derivative(Clone   (bound="T:Clone"))]
#[derivative(Default (bound="T:Default"))]
#[derivative(Debug   (bound="T:Debug"))]
pub struct WithPhantom<T, P=()> {
    #[shrinkwrap(main_field)]
    pub without_phantom: T,
    phantom: PhantomData<P>
}

impl<T, P> WithPhantom<T, P> {
    pub fn new(without_phantom: T) -> Self {
        let phantom = PhantomData;
        Self { without_phantom, phantom }
    }
}



// ==========================
// === PhantomConversions ===
// ==========================

/// A utility for easy driving of type-level computations from value level. Often we've got some
/// type level relations, like a few singleton types, and for each such type we've got an associated
/// value. For example, we can define types `Int` and `Float` and associate with them
/// `WebGlContext::Int` and `WebGlContext::Float` constants encoded as `GlEnum`. In order to convert
/// `Int` or `Float` to the `GlEnum` we do not need the instance of the types, only the information
/// what type it was. So we can define:
///
/// ```compile_fail
/// impl From<PhantomData<Int>> for u32 {
///     from(_:PhantomData<Int>>) {
///         GlEnum(WebGlContext::Int)
///     }
/// }
/// ```
///
/// And use it like:
///
/// ```compile_fail
/// let val = GlEnum::from(PhantomData::<Int>)
/// ```
///
/// Using this utility we can always write the following code instead:
///
/// ```compile_fail
/// let val = GlEnum::phantom_from::<Int>()
/// ```
pub trait PhantomConversions: Sized {
    fn phantom_into<P>() -> P where Self:PhantomInto<P> {
        PhantomData::<Self>.into()
    }
    fn phantom_from<P:PhantomInto<Self>>() -> Self {
        PhantomData::<P>.into()
    }
}
impl<T> PhantomConversions for T {}

/// Like `Into` but for phantom types.
pub trait PhantomInto<T> = where PhantomData<Self>: Into<T>;


/// Provides method `to`, which is just like `into` but allows fo superfish syntax.
pub trait ToImpl: Sized {
    fn to<P>(self) -> P where Self:Into<P> {
        self.into()
    }
}
impl<T> ToImpl for T {}



// TODO
// This impl should be hidden behind a flag. Not everybody using prelude want to import nalgebra.
impl <T,R,C,S> TypeDisplay for nalgebra::Matrix<T,R,C,S>
where T:nalgebra::Scalar, R:nalgebra::DimName, C:nalgebra::DimName {
    fn type_display() -> String {
        let cols = <C as nalgebra::DimName>::dim();
        let rows = <R as nalgebra::DimName>::dim();
        let item = type_name::<T>();
        match cols {
            1 => format!("Vector{}<{}>"    , rows, item),
            _ => format!("Matrix{}x{}<{}>" , rows, cols, item)
        }
    }
}



#[macro_export]
macro_rules! clone_boxed {
    ( $name:ident ) => { paste::item! {
        #[allow(missing_docs)]
        pub trait [<CloneBoxedFor $name>] {
            fn clone_boxed(&self) -> Box<dyn $name>;
        }

        impl<T:Clone+$name+'static> [<CloneBoxedFor $name>] for T {
            fn clone_boxed(&self) -> Box<dyn $name> {
                Box::new(self.clone())
            }
        }

        impl Clone for Box<dyn $name> {
            fn clone(&self) -> Self {
                self.clone_boxed()
            }
        }
    }}
}


// ===================
// === WithContent2 ===
// ===================

pub trait WithContent2 {
    type Content;
    fn with_content<F:FnOnce(&Self::Content)->T,T>(&self, f:F) -> T;
}

impl<T:Deref> WithContent2 for T
    where <T as Deref>::Target: WithContent2 {
    type Content = <<T as Deref>::Target as WithContent2>::Content;
    default fn with_content<F:FnOnce(&Self::Content)->R,R>(&self, f:F) -> R {
        self.deref().with_content(f)
    }
}



// =============
// === Value ===
// =============

/// Defines relation between types and values, like between `True` and `true`.
pub trait KnownTypeValue {

    /// The value-level counterpart of this type-value.
    type Value;

    /// The value of this type-value.
    fn value() -> Self::Value;
}

pub type TypeValue<T> = <T as KnownTypeValue>::Value;



// =======================
// === Type-level Bool ===
// =======================

/// Type level `true` value.
#[derive(Clone,Copy,Debug)]
pub struct True {}

/// Type level `false` value.
#[derive(Clone,Copy,Debug)]
pub struct False {}

impl KnownTypeValue for True {
    type Value = bool;
    fn value() -> Self::Value {
        true
    }
}

impl KnownTypeValue for False {
    type Value = bool;
    fn value() -> Self::Value {
        false
    }
}

/// Alias for `for<'t> &'t Self : Into<T>`.
pub trait RefInto<T> = where for<'t> &'t Self : Into<T>;



// =============
// === Owned ===
// =============

pub trait AsOwned {
    type Owned;
}

impl<T> AsOwned for &T {
    type Owned = T;
}

pub type Owned<T> = <T as AsOwned>::Owned;

pub trait IntoOwned = AsOwned + Into<Owned<Self>>;



/// Placeholder type used to represent any value. It is useful to define type-level relations like
/// defining an unit with any quantity, let it be distance or mass.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Anything {}
