#![allow(missing_docs)]

//! Metric definition.

use crate::prelude::*;

use crate::system::gpu::shader::glsl;
use crate::system::gpu::shader::glsl::Glsl;
use crate::system::gpu::shader::glsl::traits::*;

use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::Neg;
use nalgebra::*;


#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Anything{}


// ============
// === Unit ===
// ============

/// Abstraction for any unit type. It is parametrized by:
///   - Quantity, like distance, angle, or mass. See https://en.wikipedia.org/wiki/Quantity .
///   - Type, like pixels, degrees, or radians.
///   - Repr, like f32
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Unit<Quantity=Anything,Type=Anything,Repr=f32> {
    pub value : Repr,
    _quantity : PhantomData<Quantity>,
    _type     : PhantomData<Type>,
}

impl<Quantity,Type,Repr> Unit<Quantity,Type,Repr> {
    pub fn new(value:Repr) -> Self {
        let _quantity = PhantomData;
        let _type     = PhantomData;
        Self {value,_quantity,_type}
    }
}

impls! { [Quantity,Type,Repr] From<Repr> for Unit<Quantity,Type,Repr>  { |t| {Self::new(t)} } }
impls! { [Quantity,Type]      From<Unit<Quantity,Type,f32>> for f32    { |t| {t.value} } }

macro_rules! define_value_operator {
    ( $name:ident $fn:ident ($op:tt) <$rhs:ty> for $lhs:ty ) => {
        define_value_operator_rhs_ref! { $name $fn ($op) <$rhs> for $lhs [.value] }
        define_value_operator_lhs_ref! { $name $fn ($op) <$rhs> for $lhs }
    }
}

macro_rules! define_value_operator_rhs_ref {
    ( $name:ident $fn:ident ($op:tt) <$rhs:ty> for $lhs:ty [$($rhs_val:tt)*] ) => {
        impl<Quantity,Type,V,S> $name<$rhs> for $lhs
        where V:$name<S> {
            type Output = Unit<Quantity,Type,<V as $name<S>>::Output>;
            fn $fn(self, rhs:$rhs) -> Self::Output {
                (self.value $op rhs $($rhs_val)*).into()
            }
        }

        impl<'t,Quantity,Type,V,S> $name<$rhs> for &'t $lhs
        where &'t V : $name<S> {
            type Output = Unit<Quantity,Type,<&'t V as $name<S>>::Output>;
            fn $fn(self, rhs:$rhs) -> Self::Output {
                (&self.value $op rhs $($rhs_val)*).into()
            }
        }
    }
}

macro_rules! define_value_operator_lhs_ref {
    ( $name:ident $fn:ident ($op:tt) <$rhs:ty> for $lhs:ty ) => {
        impl<'t,Quantity,Type,V,S> $name<&'t $rhs> for &'t $lhs
        where &'t V : $name<&'t S> {
            type Output = Unit<Quantity,Type,<&'t V as $name<&'t S>>::Output>;
            fn $fn(self, rhs:&'t $rhs) -> Self::Output {
                (&self.value $op &rhs.value).into()
            }
        }

        impl<'t,Quantity,Type,V,S> $name<&'t $rhs> for $lhs
            where V : $name<&'t S> {
            type Output = Unit<Quantity,Type,<V as $name<&'t S>>::Output>;
            fn $fn(self, rhs:&'t $rhs) -> Self::Output {
                (self.value $op &rhs.value).into()
            }
        }
    }
}

define_value_operator! { Add add (+) <Unit<Quantity,Type,S>> for Unit<Quantity,Type,V> }
define_value_operator! { Sub sub (-) <Unit<Quantity,Type,S>> for Unit<Quantity,Type,V> }

define_value_operator_rhs_ref! { Mul mul (*) <S> for Unit<Quantity,Type,V> [] }
define_value_operator_rhs_ref! { Div div (/) <S> for Unit<Quantity,Type,V> [] }


impl<Quantity,Type> Mul<Unit<Quantity,Type,f32>> for f32 {
    type Output = Unit<Quantity,Type,f32>;
    fn mul(self, rhs:Unit<Quantity,Type,f32>) -> Self::Output {
        (self * rhs.value).into()
    }
}

impl<Quantity,Type,V> Neg for Unit<Quantity,Type,V>
    where V:Neg<Output=V> {
    type Output = Unit<Quantity,Type,V>;
    fn neg(self) -> Self::Output {
        (-self.value).into()
    }
}

impl<'t,Quantity,Type,V> Neg for &'t Unit<Quantity,Type,V>
    where &'t V : Neg {
    type Output = Unit<Quantity,Type,<&'t V as Neg>::Output>;
    fn neg(self) -> Self::Output {
        (-&self.value).into()
    }
}


pub mod quantity {
    #[derive(Clone,Copy,Debug,Eq,PartialEq)]
    pub struct Distance {}

    #[derive(Clone,Copy,Debug,Eq,PartialEq)]
    pub struct Angle {}
}



// ================
// === Distance ===
// ================



pub type Distance               = Unit<quantity::Distance>;
pub type DistanceIn<Type,V=f32> = Unit<quantity::Distance,Type,V>;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Pixels;


pub trait PixelDistance {
    fn px(&self) -> DistanceIn<Pixels>;
}

impl PixelDistance for f32 {
    fn px(&self) -> DistanceIn<Pixels> {
        DistanceIn::new(*self)
    }
}

impl PixelDistance for i32 {
    fn px(&self) -> DistanceIn<Pixels> {
        DistanceIn::new(*self as f32)
    }
}

impls! { From<DistanceIn<Pixels>> for Glsl { |t| { t.value.into() } }}
impls! { From<&DistanceIn<Pixels>> for Glsl { |t| { t.value.into() } }}

impls! { From<PhantomData<DistanceIn<Pixels>>> for glsl::PrimType {
    |_|  { PhantomData::<f32>.into() }
}}

impls! { From<PhantomData<Vector2<DistanceIn<Pixels>>>> for glsl::PrimType {
    |_|  { PhantomData::<Vector2<f32>>.into() }
}}



// =============
// === Angle ===
// =============

//pub struct AnyAngle {}



pub type Angle = Unit<quantity::Angle>;

pub type AngleIn<Type,V=f32> = Unit<quantity::Angle,Type,V>;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Degrees;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Radians;


pub trait AngleOps {
    fn degrees(&self) -> AngleIn<Degrees>;
    fn radians(&self) -> AngleIn<Radians>;

    fn deg(&self) -> AngleIn<Degrees> {
        self.degrees()
    }

    fn rad(&self) -> AngleIn<Radians> {
        self.radians()
    }
}

impl AngleOps for f32 {
    fn degrees(&self) -> AngleIn<Degrees> {
        AngleIn::new(*self)
    }

    fn radians(&self) -> AngleIn<Radians> {
        AngleIn::new(*self)
    }
}

impl AngleOps for i32 {
    fn degrees(&self) -> AngleIn<Degrees> {
        AngleIn::new(*self as f32)
    }

    fn radians(&self) -> AngleIn<Radians> {
        AngleIn::new(*self as f32)
    }
}

impls! { From< AngleIn<Radians>> for Glsl { |t| { iformat!("Radians({t.value.glsl()})").into() } }}
impls! { From<&AngleIn<Radians>> for Glsl { |t| { iformat!("Radians({t.value.glsl()})").into() } }}
impls! { From< AngleIn<Degrees>> for Glsl { |t| { iformat!("radians(Degrees({t.value.glsl()}))").into() } }}
impls! { From<&AngleIn<Degrees>> for Glsl { |t| { iformat!("radians(Degrees({t.value.glsl()}))").into() } }}

impls! { From<PhantomData<AngleIn<Radians>>> for glsl::PrimType {
    |_|  { "Radians".into() }
}}



// ==============
// === Traits ===
// ==============

pub mod traits {
    pub use super::PixelDistance;
    pub use super::AngleOps;
}


