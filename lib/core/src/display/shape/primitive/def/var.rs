//! This module defines an abstraction for all types which can be used as GLSL code values.

use crate::prelude::*;

use crate::system::gpu::shader::glsl::Glsl;
use crate::system::gpu::types::*;

use nalgebra::Scalar;

use crate::math::topology::metric::*;
use crate::data::color;
use crate::data::color::*;

use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::Neg;



pub trait Pxx {
    fn pxx(&self) -> ShapeData<DistanceIn<Pixels>>;
}

impl<T:DistanceOps> Pxx for T {
    fn pxx(&self) -> ShapeData<DistanceIn<Pixels>> {
        self.px().into()
    }

}


pub mod traits {
    pub use super::Pxx;
}


//// ==================
//// === ShaderData ===
//// ==================

pub trait ValidInput<T> = ValidInputMarker<T> + Into<Glsl>;

pub trait ValidInputMarker<T> {}
pub trait ValidInputMarkerNested<T> {}


// === Instances ===

impl<T> ValidInputMarker<ShapeData<T>> for Glsl    {}
impl<T> ValidInputMarker<ShapeData<T>> for &Glsl   {}
impl<T> ValidInputMarker<ShapeData<T>> for String  {}
impl<T> ValidInputMarker<ShapeData<T>> for &String {}
impl<T> ValidInputMarker<ShapeData<T>> for &str    {}
//impl<T> ValidInputMarker<ShapeData<T>> for ShapeData<T>  {}
//impl<T> ValidInputMarker<ShapeData<T>> for &ShapeData<T> {}
impl<T> ValidInputMarker<ShapeData<T>> for  T      {}
impl<T> ValidInputMarker<ShapeData<T>> for &T      {}

impl<E1,E2,T> ValidInputMarker<ShapeData<Rgba<E1,T>>> for Rgb<E2,T>
    where E1:color::RgbStandard, E2:color::RgbStandard, T:color::Component {}

impl<E1,E2,T> ValidInputMarker<ShapeData<Rgba<E1,T>>> for Rgba<E2,T>
    where E1:color::RgbStandard, E2:color::RgbStandard, T:color::Component {}

impl<E,T,G> ValidInputMarker<ShapeData<Rgba<E,T>>> for DistanceGradient<G>
    where E:color::RgbStandard, T:color::Component {}

impl<T,U,V> ValidInputMarker<ShapeData<Value<T,Unknown,V>>> for Value<T,U,V> where {}

impl<T,S1,S2> ValidInputMarker<ShapeData<Vector2<T>>> for (S1,S2)
    where T:Scalar, S1:ValidInputMarkerNested<ShapeData<T>>, S2:ValidInputMarkerNested<ShapeData<T>> {}


impl<T,S> ValidInputMarkerNested<T> for S where S:ValidInputMarker<T> {}
impl<T> ValidInputMarkerNested<ShapeData<T>> for ShapeData<T> {}
impl<T> ValidInputMarkerNested<ShapeData<T>> for &ShapeData<T> {}


// ==================
// === ShaderData ===
// ==================

#[derive(Clone,Debug)]
pub enum ShapeData<T> {
    Static  (T),
    Dynamic (Glsl),
}

impls! {[T:Clone] From<&ShapeData<T>> for ShapeData<T> { |t| { t.clone () } }}

impls! {[T:RefInto<Glsl>] From<&ShapeData<T>> for Glsl { |t|
    match t {
        ShapeData::Static  (s) => s.into(),
        ShapeData::Dynamic (s) => s.into(),
    }
}}

impls! {[T:Into<Glsl>] From<ShapeData<T>> for Glsl { |t|
    match t {
        ShapeData::Static  (s) => s.into(),
        ShapeData::Dynamic (s) => s.into(),
    }
}}

impl<T,S> From<T> for ShapeData<S>
where T : ValidInput<ShapeData<S>>,
      S : ValidInput<ShapeData<S>> {
    default fn from(t:T) -> Self {
        Self::Dynamic(t.into())
    }
}

impl<T> From<T> for ShapeData<T>
where T : ValidInput<ShapeData<T>> {
    fn from(t:T) -> Self {
        Self::Static(t)
    }
}


// === Operators ===

macro_rules! define_operator_newtype {
    ( $name:ident $fn:ident $base:ident where [$($bounds:tt)*] {
        |$v_lhs:ident, $v_rhs:ident| $($body:tt)*
    } ) => {
        impl<'t,B,A> $name<&'t $base<B>> for &'t $base<A>
        where &'t A : $name<&'t B>, $($bounds)* {
            type Output = $base<<&'t A as $name<&'t B>>::Output>;
            fn $fn(self, rhs:&'t $base<B>) -> Self::Output {
                let f = move |$v_lhs:&'t $base<A>, $v_rhs:&'t $base<B>| { $($body)* };
                f(self,rhs)
            }
        }

        impl<'t,B,A> $name<&'t $base<B>> for $base<A>
        where A : $name<&'t B>, $($bounds)* {
            type Output = $base<<A as $name<&'t B>>::Output>;
            fn $fn(self, rhs:&'t $base<B>) -> Self::Output {
                let f = move |$v_lhs:$base<A>, $v_rhs:&'t $base<B>| { $($body)* };
                f(self,rhs)
            }
        }

        impl<'t,B,A> $name<$base<B>> for &'t $base<A>
        where &'t A : $name<B>, $($bounds)* {
            type Output = $base<<&'t A as $name<B>>::Output>;
            fn $fn(self, rhs:$base<B>) -> Self::Output {
                let f = move |$v_lhs:&'t $base<A>, $v_rhs:$base<B>| { $($body)* };
                f(self,rhs)
            }
        }

        impl<B,A> $name<$base<B>> for $base<A>
        where A : $name<B>, $($bounds)* {
            type Output = $base<<A as $name<B>>::Output>;
            fn $fn(self, rhs:$base<B>) -> Self::Output {
                let f = move |$v_lhs:$base<A>, $v_rhs:$base<B>| { $($body)* };
                f(self,rhs)
            }
        }
    }
}

macro_rules! define_shape_data_operator {
    ( $name:ident $fn:ident ($opr:tt) where $bounds:tt ) => {
        define_operator_newtype! { $name $fn ShapeData where $bounds {
            |lhs,rhs| {
                match lhs {
                    ShapeData::Static(lhs) => match rhs {
                        ShapeData::Static(rhs) => ShapeData::Static(lhs $opr rhs),
                        _ => ShapeData::Dynamic(iformat! {
                            "({lhs.glsl()} {stringify!($opr)} {rhs.glsl()})"
                        }.into())
                    },
                    _ => ShapeData::Dynamic(iformat! {
                        "({lhs.glsl()} {stringify!($opr)} {rhs.glsl()})"
                    }.into())
                }
            }
        }}
    }
}

macro_rules! define_shape_data_prim_operator {
    ( $name:ident $fn:ident ($opr:tt) for $target:ident where [$($bounds:tt)*] ) => {
        impl<A> $name<$target> for ShapeData<A>
        where A: $name<$target>, $($bounds)* {
            type Output = ShapeData<<A as $name<$target>>::Output>;
            default fn $fn(self, rhs: $target) -> Self::Output {
                let f = move |lhs: ShapeData<A>, rhs: $target| {
                    match lhs {
                        ShapeData::Static(lhs) => ShapeData::Static(lhs $opr rhs),
                        _ => ShapeData::Dynamic(iformat! {
                            "({lhs.glsl()} {stringify!($opr)} {rhs.glsl()})"
                        }.into())
                    }
                };
                f(self, rhs)
            }
        }

        impl<'t,A> $name<$target> for &'t ShapeData<A>
        where &'t A: $name<$target>, $($bounds)* {
            type Output = ShapeData<<&'t A as $name<$target>>::Output>;
            default fn $fn(self, rhs: $target) -> Self::Output {
                let f = move |lhs: &'t ShapeData<A>, rhs: $target| {
                    match lhs {
                        ShapeData::Static(lhs) => ShapeData::Static(lhs $opr rhs),
                        _ => ShapeData::Dynamic(iformat! {
                            "({lhs.glsl()} {stringify!($opr)} {rhs.glsl()})"
                        }.into())
                    }
                };
                f(self, rhs)
            }
        }
    }
}

define_shape_data_operator!      { Add add (+)         where [A:RefInto<Glsl>, B:RefInto<Glsl>] }
define_shape_data_operator!      { Sub sub (-)         where [A:RefInto<Glsl>, B:RefInto<Glsl>] }
define_shape_data_operator!      { Mul mul (*)         where [A:RefInto<Glsl>, B:RefInto<Glsl>] }
define_shape_data_operator!      { Div div (/)         where [A:RefInto<Glsl>, B:RefInto<Glsl>] }
define_shape_data_prim_operator! { Div div (/) for f32 where [A:RefInto<Glsl>] }
define_shape_data_prim_operator! { Mul mul (*) for f32 where [A:RefInto<Glsl>] }

impl<T> Neg for ShapeData<T>
where T : Neg + RefInto<Glsl> {
    type Output = ShapeData<<T as Neg>::Output>;
    fn neg(self) -> Self::Output {
        match self {
            ShapeData::Static(t)  => ShapeData::Static(-t),
            ShapeData::Dynamic(t) => ShapeData::Dynamic(iformat!("(-{t})").into()),
        }
    }
}
