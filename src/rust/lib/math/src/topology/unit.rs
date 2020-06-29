//! Defines unit of measurement abstraction. See: https://en.wikipedia.org/wiki/Unit_of_measurement

use crate::algebra::*;

use std::ops::*;
use std::marker::PhantomData;



// ============
// === Unit ===
// ============

/// Abstraction for any unit type parameterized by a type (like distance in pixels) and underlying
/// numerical representation.
#[derive(Debug,PartialEq)]
pub struct Unit<Type,Repr=f32> {
    /// The raw value of this unit.
    pub value : Repr,
    _type     : PhantomData<Type>,
}

impl<Type,Repr:Copy>  Copy  for Unit<Type,Repr> {}
impl<Type,Repr:Clone> Clone for Unit<Type,Repr> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}

impl<Type,Repr> Unit<Type,Repr> {
    /// Constructor.
    pub fn new(value:Repr) -> Self {
        let _type = PhantomData;
        Self {value,_type}
    }
}


// === Conversions ===

impl<Type,Repr> From<Repr> for Unit<Type,Repr> {
    fn from(t:Repr) -> Self {
        Self::new(t)
    }
}

impl<Type,Repr:Clone> From<&Repr> for Unit<Type,Repr> {
    fn from(t:&Repr) -> Self {
        Self::new(t.clone())
    }
}

impl<Type> From<Unit<Type,f32>> for f32 {
    fn from(t:Unit<Type,f32>) -> Self {
        t.value
    }
}

impl<Type> From<&Unit<Type,f32>> for f32 {
    fn from(t:&Unit<Type,f32>) -> Self {
        t.value
    }
}



// =================
// === Operators ===
// =================

// === Unit x Repr -> Unit Operators ===

macro_rules! impl_opr_unit_x_repr_to_unit {
    ( $name:ident $fn:ident $t:ident ) => {
        impl<Type> $name<$t> for Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:$t) -> Self::Output {
                (self.value.$fn(rhs)).into()
            }
        }

        impl<Type> $name<$t> for &Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:$t) -> Self::Output {
                (self.value.$fn(rhs)).into()
            }
        }

        impl<Type> $name<&$t> for Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:&$t) -> Self::Output {
                (self.value.$fn(*rhs)).into()
            }
        }

        impl<Type> $name<&$t> for &Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:&$t) -> Self::Output {
                ((&self.value).$fn(*rhs)).into()
            }
        }
    }
}

macro_rules! impl_opr_repr_x_unit_to_unit {
    ( $name:ident $fn:ident $t:ident ) => {
        impl<Type> $name<Unit<Type,$t>> for $t {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:Unit<Type,$t>) -> Self::Output {
                (self.$fn(rhs.value)).into()
            }
        }

        impl<Type> $name<Unit<Type,$t>> for &$t {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:Unit<Type,$t>) -> Self::Output {
                (self.$fn(rhs.value)).into()
            }
        }

        impl<Type> $name<&Unit<Type,$t>> for $t {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:&Unit<Type,$t>) -> Self::Output {
                (self.$fn(rhs.value)).into()
            }
        }

        impl<Type> $name<&Unit<Type,$t>> for &$t {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:&Unit<Type,$t>) -> Self::Output {
                (self.$fn(rhs.value)).into()
            }
        }
    }
}


// === Unit x Unit -> Unit Operators ===

macro_rules! impl_opr_unit_x_unit_to_unit {
    ( $name:ident $fn:ident $t:ident ) => {
        impl<Type> $name<Unit<Type,$t>> for Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:Unit<Type,$t>) -> Self::Output {
                (self.value.$fn(rhs.value)).into()
            }
        }

        impl<Type> $name<Unit<Type,$t>> for &Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:Unit<Type,$t>) -> Self::Output {
                (self.value.$fn(rhs.value)).into()
            }
        }

        impl<Type> $name<&Unit<Type,$t>> for Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:&Unit<Type,$t>) -> Self::Output {
                (self.value.$fn(rhs.value)).into()
            }
        }

        impl<Type> $name<&Unit<Type,$t>> for &Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self, rhs:&Unit<Type,$t>) -> Self::Output {
                (self.value.$fn(rhs.value)).into()
            }
        }
    }
}


// === Unit x Unit -> Repr Operators ===

macro_rules! impl_opr_unit_x_unit_to_repr {
    ( $name:ident $fn:ident $t:ident ) => {
        impl<Type> $name<Unit<Type,$t>> for Unit<Type,$t> {
            type Output = $t;
            fn $fn(self, rhs:Unit<Type,$t>) -> Self::Output {
                self.value.$fn(rhs.value)
            }
        }

        impl<Type> $name<Unit<Type,$t>> for &Unit<Type,$t> {
            type Output = $t;
            fn $fn(self, rhs:Unit<Type,$t>) -> Self::Output {
                self.value.$fn(rhs.value)
            }
        }

        impl<Type> $name<&Unit<Type,$t>> for Unit<Type,$t> {
            type Output = $t;
            fn $fn(self, rhs:&Unit<Type,$t>) -> Self::Output {
                self.value.$fn(rhs.value)
            }
        }

        impl<Type> $name<&Unit<Type,$t>> for &Unit<Type,$t> {
            type Output = $t;
            fn $fn(self, rhs:&Unit<Type,$t>) -> Self::Output {
                self.value.$fn(rhs.value)
            }
        }
    }
}


// === Unit -> Unit Operators ===

macro_rules! impl_opr_unit_to_unit {
    ( $name:ident $fn:ident $t:ident ) => {
        impl<Type> $name for Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self) -> Self::Output {
                self.value.$fn().into()
            }
        }

        impl<Type> $name for &Unit<Type,$t> {
            type Output = Unit<Type,$t>;
            fn $fn(self) -> Self::Output {
                self.value.$fn().into()
            }
        }
    }
}

impl<Type,Repr> Abs for Unit<Type,Repr> where Repr:Abs {
    fn abs(&self) -> Self {
        Self { value:self.value.abs(), ..*self }
    }
}


// === Implementations ===

impl_opr_unit_x_repr_to_unit! (Div div f32);
impl_opr_unit_x_repr_to_unit! (Mul mul f32);
impl_opr_repr_x_unit_to_unit! (Mul mul f32);
impl_opr_unit_x_unit_to_unit! (Sub sub f32);
impl_opr_unit_x_unit_to_unit! (Add add f32);
impl_opr_unit_x_unit_to_repr! (Div div f32);
impl_opr_unit_to_unit!        (Neg neg f32);

impl_opr_unit_x_repr_to_unit! (Div div usize);
impl_opr_unit_x_repr_to_unit! (Mul mul usize);
impl_opr_repr_x_unit_to_unit! (Mul mul usize);
impl_opr_unit_x_unit_to_unit! (Sub sub usize);
impl_opr_unit_x_unit_to_unit! (Add add usize);
impl_opr_unit_x_unit_to_unit! (SaturatingAdd saturating_add usize);
impl_opr_unit_x_unit_to_repr! (Div div usize);



// ==================
// === Prim Units ===
// ==================

#[macro_export]
macro_rules! define_unit {
    ( $name:ident, $tp:ident, $type_name:ident, $trait_name:ident, $f:ident ) => {
        #[derive(Clone,Copy,Debug,Eq,PartialEq)]
        pub struct $type_name;

        pub type $name<Repr=$tp> = Unit<$type_name,Repr>;

        pub fn $name(value:$tp) -> $name {
            $name::new(value)
        }

        pub trait $trait_name {
            type Output;
            fn $f(&self) -> Self::Output;
        }

        impl $trait_name for $tp {
            type Output = $name;
            fn $f(&self) -> Self::Output {
                $name::new(*self)
            }
        }

        // FIXME this impl is non-uniform
        impl $trait_name for i32 {
            type Output = $name;
            fn $f(&self) -> Self::Output {
                $name::new(*self as $tp)
            }
        }

        impl $trait_name for Vector2<$tp> {
            type Output = Vector2<$name>;
            fn $f(&self) -> Self::Output {
                Vector2($name::new(self.x),$name::new(self.y))
            }
        }

    }
}

define_unit!(Pixels  , f32 , PixelsType  , ToPixels  , px);
define_unit!(Radians , f32 , RadiansType , ToRadians , radians);
define_unit!(Degrees , f32 , DegreesType , ToDegrees , degrees);


// === Conversions ===

impl ToRadians for Degrees {
    type Output = Radians;
    fn radians(&self) -> Self::Output {
        Radians::new(std::f32::consts::PI * self.value / 180.0)
    }
}

impl ToDegrees for Radians {
    type Output = Degrees;
    fn degrees(&self) -> Self::Output {
        Degrees::new(180.0 * self.value / std::f32::consts::PI)
    }
}



// ==============
// === Traits ===
// ==============

/// Commonly used traits.
pub mod traits {
    pub use super::ToPixels;
    pub use super::ToRadians;
    pub use super::ToDegrees;
}
