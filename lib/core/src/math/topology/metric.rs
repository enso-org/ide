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



// =============
// === Value ===
// =============

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Unknown{}

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Value<Tp=Unknown,Unit=Unknown,V=f32> {
    pub value : V,
    _type     : PhantomData<Tp>,
    _unit     : PhantomData<Unit>,
}

impl<Tp,Unit,V> Value<Tp,Unit,V> {
    pub fn new(value:V) -> Self {
        let _type = PhantomData;
        let _unit = PhantomData;
        Self {value,_type,_unit}
    }
}

impls! { [Tp,Unit,V] From<V>                  for Value<Tp,Unit,V> { |t| {Self::new(t)} } }
impls! { [Tp,Unit]   From<Value<Tp,Unit,f32>> for f32              { |t| {t.value} } }

macro_rules! define_value_operator {
    ( $name:ident $fn:ident ($op:tt) <$rhs:ty> for $lhs:ty ) => {
        define_value_operator_rhs_ref! { $name $fn ($op) <$rhs> for $lhs [.value] }
        define_value_operator_lhs_ref! { $name $fn ($op) <$rhs> for $lhs }
    }
}

macro_rules! define_value_operator_rhs_ref {
    ( $name:ident $fn:ident ($op:tt) <$rhs:ty> for $lhs:ty [$($rhs_val:tt)*] ) => {
        impl<Tp,Unit,V,S> $name<$rhs> for $lhs
        where V:$name<S> {
            type Output = Value<Tp,Unit,<V as $name<S>>::Output>;
            fn $fn(self, rhs:$rhs) -> Self::Output {
                (self.value $op rhs $($rhs_val)*).into()
            }
        }

        impl<'t,Tp,Unit,V,S> $name<$rhs> for &'t $lhs
        where &'t V : $name<S> {
            type Output = Value<Tp,Unit,<&'t V as $name<S>>::Output>;
            fn $fn(self, rhs:$rhs) -> Self::Output {
                (&self.value $op rhs $($rhs_val)*).into()
            }
        }
    }
}

macro_rules! define_value_operator_lhs_ref {
    ( $name:ident $fn:ident ($op:tt) <$rhs:ty> for $lhs:ty ) => {
        impl<'t,Tp,Unit,V,S> $name<&'t $rhs> for &'t $lhs
        where &'t V : $name<&'t S> {
            type Output = Value<Tp,Unit,<&'t V as $name<&'t S>>::Output>;
            fn $fn(self, rhs:&'t $rhs) -> Self::Output {
                (&self.value $op &rhs.value).into()
            }
        }

        impl<'t,Tp,Unit,V,S> $name<&'t $rhs> for $lhs
            where V : $name<&'t S> {
            type Output = Value<Tp,Unit,<V as $name<&'t S>>::Output>;
            fn $fn(self, rhs:&'t $rhs) -> Self::Output {
                (self.value $op &rhs.value).into()
            }
        }
    }
}

define_value_operator! { Add add (+) <Value<Tp,Unit,S>> for Value<Tp,Unit,V> }
define_value_operator! { Sub sub (-) <Value<Tp,Unit,S>> for Value<Tp,Unit,V> }

define_value_operator_rhs_ref! { Mul mul (*) <S> for Value<Tp,Unit,V> [] }
define_value_operator_rhs_ref! { Div div (/) <S> for Value<Tp,Unit,V> [] }


impl<Tp,Unit> Mul<Value<Tp,Unit,f32>> for f32 {
    type Output = Value<Tp,Unit,f32>;
    fn mul(self, rhs:Value<Tp,Unit,f32>) -> Self::Output {
        (self * rhs.value).into()
    }
}

impl<Tp,Unit,V> Neg for Value<Tp,Unit,V>
    where V:Neg<Output=V> {
    type Output = Value<Tp,Unit,V>;
    fn neg(self) -> Self::Output {
        (-self.value).into()
    }
}

impl<'t,Tp,Unit,V> Neg for &'t Value<Tp,Unit,V>
    where &'t V : Neg {
    type Output = Value<Tp,Unit,<&'t V as Neg>::Output>;
    fn neg(self) -> Self::Output {
        (-&self.value).into()
    }
}



// ================
// === Distance ===
// ================

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct DistanceValue {}

pub type Distance               = Value<DistanceValue>;
pub type DistanceIn<Unit,V=f32> = Value<DistanceValue,Unit,V>;

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

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct AngleValue {}

pub type Angle = Value<AngleValue>;

pub type AngleIn<Unit,V=f32> = Value<AngleValue,Unit,V>;

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


