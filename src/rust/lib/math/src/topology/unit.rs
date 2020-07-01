//! Defines unit of measurement abstraction. See: https://en.wikipedia.org/wiki/Unit_of_measurement

use crate::algebra::*;

use std::ops::*;



// ==============
// === Macros ===
// ==============

#[macro_export]
macro_rules! unit {
    ($(#$meta:tt)* $name:ident :: $vname:ident (f32)) => {
        $crate::signed_unit_float_like!{$(#$meta)* $name :: $vname (f32)}
    };
    ($(#$meta:tt)* $name:ident :: $vname:ident (f64)) => {
        $crate::signed_unit_float_like!{$(#$meta)* $name :: $vname (f64)}
    };
    ($(#$meta:tt)* $name:ident :: $vname:ident (usize)) => {
        $crate::unsigned_unit!{$(#$meta)* $name :: $vname (usize)}
    };
    ($(#$meta:tt)* $name:ident :: $vname:ident (u32)) => {
        $crate::unsigned_unit!{$(#$meta)* $name :: $vname (u32)}
    };
    ($(#$meta:tt)* $name:ident :: $vname:ident (u64)) => {
        $crate::unsigned_unit!{$(#$meta)* $name :: $vname (u64)}
    };
    ($(#$meta:tt)* $name:ident :: $vname:ident (i32)) => {
        $crate::signed_unit!{$(#$meta)* $name :: $vname (i32)}
    };
    ($(#$meta:tt)* $name:ident :: $vname:ident (i64)) => {
        $crate::signed_unit!{$(#$meta)* $name :: $vname (i64)}
    };
}

#[macro_export]
macro_rules! unsigned_unit {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub sub $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add add $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {SaturatingAdd saturating_add $name}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Div div $name $field_type}
            $crate::impl_FIELD_x_UNIT_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_UNIT_to_FIELD! {Div div $name $field_type}
            $crate::impl_UNIT_x_UNIT!          {AddAssign add_assign $name}

            pub trait Into {
                type Output;
                fn $vname(self) -> Self::Output;
            }

            impl<T:std::convert::Into<$name>> Into for T {
                type Output = $name;
                fn $vname(self) -> Self::Output {
                    self.into()
                }
            }

            pub mod export {
                pub use super::$name;
                pub use super::Into as TRAIT_Into;
            }
        }
        pub use $vname::export::*;
    };
}

#[macro_export]
macro_rules! unsigned_unit_proxy {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub sub $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add add $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {SaturatingAdd saturating_add $name}
//            $crate::impl_UNIT_x_FIELD_to_UNIT! {Mul mul $name $field_type}
//            $crate::impl_UNIT_x_FIELD_to_UNIT! {Div div $name $field_type}
//            $crate::impl_FIELD_x_UNIT_to_UNIT! {Mul mul $name $field_type}
//            $crate::impl_UNIT_x_UNIT_to_FIELD! {Div div $name $field_type}
            $crate::impl_UNIT_x_UNIT!          {AddAssign add_assign $name}

            pub trait Into {
                type Output;
                fn $vname(self) -> Self::Output;
            }

            impl<T:std::convert::Into<$name>> Into for T {
                type Output = $name;
                fn $vname(self) -> Self::Output {
                    self.into()
                }
            }

            pub mod export {
                pub use super::$name;
                pub use super::Into as TRAIT_Into;
            }
        }
        pub use $vname::export::*;
    };
}

#[macro_export]
macro_rules! unsigned_unit_float_like {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct_float_like! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub sub $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add add $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {SaturatingAdd saturating_add $name}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Div div $name $field_type}
            $crate::impl_FIELD_x_UNIT_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_UNIT_to_FIELD! {Div div $name $field_type}
            $crate::impl_UNIT_x_UNIT!          {AddAssign add_assign $name}

            pub trait Into {
                type Output;
                fn $vname(self) -> Self::Output;
            }

            impl<T:std::convert::Into<$name>> Into for T {
                type Output = $name;
                fn $vname(self) -> Self::Output {
                    self.into()
                }
            }

            pub mod export {
                pub use super::$name;
                pub use super::Into as TRAIT_Into;
            }
        }
        pub use $vname::export::*;
    };
}

#[macro_export]
macro_rules! signed_unit {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub sub $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add add $name}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Div div $name $field_type}
            $crate::impl_FIELD_x_UNIT_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_UNIT_to_FIELD! {Div div $name $field_type}
            $crate::impl_UNIT_x_UNIT!          {AddAssign add_assign $name}
            $crate::impl_UNIT_to_UNIT!         {Neg neg $name}

            pub trait Into {
                type Output;
                fn $vname(self) -> Self::Output;
            }

            impl<T:std::convert::Into<$name>> Into for T {
                type Output = $name;
                fn $vname(self) -> Self::Output {
                    self.into()
                }
            }

            pub mod export {
                pub use super::$name;
                pub use super::Into as TRAIT_Into;
            }
        }
        pub use $vname::export::*;
    };
}

#[macro_export]
macro_rules! signed_unit_float_like {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct_float_like! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub sub $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add add $name}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Div div $name $field_type}
            $crate::impl_FIELD_x_UNIT_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_UNIT_to_FIELD! {Div div $name $field_type}
            $crate::impl_UNIT_x_UNIT!          {AddAssign add_assign $name}
            $crate::impl_UNIT_to_UNIT!         {Neg neg $name}

            pub trait Into {
                type Output;
                fn $vname(self) -> Self::Output;
            }

            impl<T:std::convert::Into<$name>> Into for T {
                type Output = $name;
                fn $vname(self) -> Self::Output {
                    self.into()
                }
            }

            pub mod export {
                pub use super::$name;
                pub use super::Into as TRAIT_Into;
            }
        }
        pub use $vname::export::*;
    };
}

#[macro_export]
macro_rules! newtype {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        use std::ops::AddAssign;

        $crate::newtype_struct! {$(#$meta)* $name { $($field : $field_type),*}}

        $crate::impl_T_x_T_to_T! {Sub           sub            $name {$($field),*}}
        $crate::impl_T_x_T_to_T! {Add           add            $name {$($field),*}}
        $crate::impl_T_x_T_to_T! {SaturatingAdd saturating_add $name {$($field),*}}

        $crate::impl_T_x_FIELD_to_T! {Sub           sub            $name {$($field:$field_type),*}}
        $crate::impl_T_x_FIELD_to_T! {Add           add            $name {$($field:$field_type),*}}
        $crate::impl_T_x_FIELD_to_T! {SaturatingAdd saturating_add $name {$($field:$field_type),*}}

        impl AddAssign<$name> for $name {
            fn add_assign(&mut self, rhs:Self) {
                *self = Self { $($field:self.$field.add(rhs.$field)),* }
            }
        }
    };
}

#[macro_export]
macro_rules! newtype_struct {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $crate::newtype_struct_def!   {$(#$meta)* $name { $($field : $field_type),*}}
        $crate::newtype_struct_impls! {$(#$meta)* $name { $($field : $field_type),*}}
    }
}

#[macro_export]
macro_rules! newtype_struct_float_like {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $crate::newtype_struct_def_float_like! {$(#$meta)* $name { $($field : $field_type),*}}
        $crate::newtype_struct_impls!          {$(#$meta)* $name { $($field : $field_type),*}}
    }
}

#[macro_export]
macro_rules! newtype_struct_def {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $(#$meta)*
        #[derive(Clone,Copy,Debug,Default,Eq,Hash,Ord,PartialEq,PartialOrd)]
        pub struct $name { $(pub $field : $field_type),* }
    }
}

#[macro_export]
macro_rules! newtype_struct_def_float_like {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $(#$meta)*
        #[derive(Clone,Copy,Debug,Default,PartialEq,PartialOrd)]
        pub struct $name { $(pub $field : $field_type),* }
    }
}

#[macro_export]
macro_rules! newtype_struct_impls {
    ($(#$meta:tt)* $name:ident { $field:ident : $field_type:ty $(,)? }) => {
        /// Smart constructor.
        $(#$meta)*
        #[allow(non_snake_case)]
        pub fn $name($field:$field_type) -> $name { $name {$field} }

        impl From<&$name>  for $name { fn from(t:&$name)  -> Self { *t } }
        impl From<&&$name> for $name { fn from(t:&&$name) -> Self { **t } }

        impl From<$name>   for $field_type { fn from(t:$name)   -> Self { t.$field } }
        impl From<&$name>  for $field_type { fn from(t:&$name)  -> Self { t.$field } }
        impl From<&&$name> for $field_type { fn from(t:&&$name) -> Self { t.$field } }

        impl From<$field_type>   for $name { fn from(t:$field_type)   -> Self { $name(t) } }
        impl From<&$field_type>  for $name { fn from(t:&$field_type)  -> Self { $name(*t) } }
        impl From<&&$field_type> for $name { fn from(t:&&$field_type) -> Self { $name(**t) } }
    };

    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        /// Smart constructor.
        $(#$meta)*
        #[allow(non_snake_case)]
        pub fn $name($($field:$field_type),*) -> $name { $name {$($field),*} }

        impl From<&$name>  for $name { fn from(t:&$name)  -> Self { *t } }
        impl From<&&$name> for $name { fn from(t:&&$name) -> Self { **t } }
        $(
            impl From<$name>   for $field_type { fn from(t:$name)   -> Self { t.$field } }
            impl From<&$name>  for $field_type { fn from(t:&$name)  -> Self { t.$field } }
            impl From<&&$name> for $field_type { fn from(t:&&$name) -> Self { t.$field } }
        )*
    };
}





// ==================
// === T x T -> T ===
// ==================

#[macro_export]
macro_rules! impl_UNIT_x_UNIT_to_UNIT {
    ($opr:ident $f:ident $name:ident) => {
        $crate::impl_T_x_T_to_T! {$opr $f $name {value}}
    }
}

#[macro_export]
macro_rules! impl_T_x_T_to_T {
    ($opr:ident $f:ident $name:ident { $($field:ident),* $(,)? }) => {
        impl $opr<$name> for $name {
            type Output = $name;
            fn $f(self, rhs:$name) -> Self::Output {
                $(let $field = self.$field.$f(rhs.$field);)*
                $name { $($field),* }
            }
        }

        impl $opr<$name> for &$name {
            type Output = $name;
            fn $f(self, rhs:$name) -> Self::Output {
                $(let $field = self.$field.$f(rhs.$field);)*
                $name { $($field),* }
            }
        }

        impl $opr<&$name> for $name {
            type Output = $name;
            fn $f(self, rhs:&$name) -> Self::Output {
                $(let $field = self.$field.$f(rhs.$field);)*
                $name { $($field),* }
            }
        }

        impl $opr<&$name> for &$name {
            type Output = $name;
            fn $f(self, rhs:&$name) -> Self::Output {
                $(let $field = self.$field.$f(rhs.$field);)*
                $name { $($field),* }
            }
        }
    };
}



// ======================
// === T x FIELD -> T ===
// ======================

#[macro_export]
macro_rules! impl_UNIT_x_FIELD_to_UNIT {
    ($opr:ident $f:ident $name:ident $field_type:ty) => {
        $crate::impl_T_x_FIELD_to_T! {$opr $f $name {value : $field_type}}
    }
}

#[macro_export]
macro_rules! impl_T_x_FIELD_to_T {
    ($opr:ident $f:ident $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $opr<$field_type> for $name {
            type Output = $name;
            fn $f(self, rhs:$field_type) -> Self::Output {
                $name { $field:self.$field.$f(rhs), ..self }
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<$field_type> for &$name {
            type Output = $name;
            fn $f(self, rhs:$field_type) -> Self::Output {
                $name { $field:self.$field.$f(rhs), ..*self }
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<&$field_type> for $name {
            type Output = $name;
            fn $f(self, rhs:&$field_type) -> Self::Output {
                $name { $field:self.$field.$f(*rhs), ..self }
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<&$field_type> for &$name {
            type Output = $name;
            fn $f(self, rhs:&$field_type) -> Self::Output {
                $name { $field:self.$field.$f(*rhs), ..*self }
            }
        }
    )*};
}


// ======================
// === FIELD x T -> T ===
// ======================

#[macro_export]
macro_rules! impl_FIELD_x_UNIT_to_UNIT {
    ($opr:ident $f:ident $name:ident $field_type:ty) => {
        $crate::impl_FIELD_x_T_to_T! {$opr $f $name {value : $field_type}}
    }
}

#[macro_export]
macro_rules! impl_FIELD_x_T_to_T {
    ($opr:ident $f:ident $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $opr<$name> for $field_type {
            type Output = $name;
            fn $f(self, rhs:$name) -> Self::Output {
                $name { $field:self.$f(rhs.$field), ..rhs }
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<$name> for &$field_type {
            type Output = $name;
            fn $f(self, rhs:$name) -> Self::Output {
                $name { $field:self.$f(rhs.$field), ..rhs }
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<&$name> for $field_type {
            type Output = $name;
            fn $f(self, rhs:&$name) -> Self::Output {
                $name { $field:self.$f(rhs.$field), ..*rhs }
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<&$name> for &$field_type {
            type Output = $name;
            fn $f(self, rhs:&$name) -> Self::Output {
                $name { $field:self.$f(rhs.$field), ..*rhs }
            }
        }
    )*};
}



// ======================
// === T x T -> FIELD ===
// ======================

#[macro_export]
macro_rules! impl_UNIT_x_UNIT_to_FIELD {
    ($opr:ident $f:ident $name:ident $field_type:ty) => {
        $crate::impl_T_x_T_to_FIELD! {$opr $f $name {value : $field_type}}
    }
}

#[macro_export]
macro_rules! impl_T_x_T_to_FIELD {
    ($opr:ident $f:ident $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {$(
        impl $opr<$name> for $name {
            type Output = $field_type;
            fn $f(self, rhs:$name) -> Self::Output {
                self.$field.$f(rhs.$field)
            }
        }

        impl $opr<$name> for &$name {
            type Output = $field_type;
            fn $f(self, rhs:$name) -> Self::Output {
                self.$field.$f(rhs.$field)
            }
        }

        impl $opr<&$name> for $name {
            type Output = $field_type;
            fn $f(self, rhs:&$name) -> Self::Output {
                self.$field.$f(rhs.$field)
            }
        }

        impl $opr<&$name> for &$name {
            type Output = $field_type;
            fn $f(self, rhs:&$name) -> Self::Output {
                self.$field.$f(rhs.$field)
            }
        }
    )*};
}



// ==============
// === T -> T ===
// ==============

#[macro_export]
macro_rules! impl_UNIT_to_UNIT {
    ($opr:ident $f:ident $name:ident) => {
        $crate::impl_T_to_T! {$opr $f $name {value}}
    }
}

#[macro_export]
macro_rules! impl_T_to_T {
    ($opr:ident $f:ident $name:ident { $($field:ident),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $opr for $name {
            type Output = $name;
            fn $f(self) -> Self::Output {
                $name { $field:self.$field.$f(), ..self }
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr for &$name {
            type Output = $name;
            fn $f(self) -> Self::Output {
                $name { $field:self.$field.$f(), ..*self }
            }
        }
    )*};
}




// =============
// === T x T ===
// =============

#[macro_export]
macro_rules! impl_UNIT_x_UNIT {
    ($opr:ident $f:ident $name:ident) => {
        $crate::impl_T_x_T! {$opr $f $name {value}}
    }
}

#[macro_export]
macro_rules! impl_T_x_T {
    ($opr:ident $f:ident $name:ident { $($field:ident),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $opr<$name> for $name {
            fn $f(&mut self, rhs:$name) {
                self.$field.$f(rhs.$field)
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<$name> for &mut $name {
            fn $f(&mut self, rhs:$name) {
                self.$field.$f(rhs.$field)
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<&$name> for $name {
            fn $f(&mut self, rhs:&$name) {
                self.$field.$f(rhs.$field)
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<&$name> for &mut $name {
            fn $f(&mut self, rhs:&$name) {
                self.$field.$f(rhs.$field)
            }
        }
    )*};
}



// =================
// === T x FIELD ===
// =================

#[macro_export]
macro_rules! impl_UNIT_x_FIELD {
    ($opr:ident $f:ident $name:ident $field_type:ty) => {
        $crate::impl_T_x_FIELD! {$opr $f $name {value : $field_type}}
    }
}

#[macro_export]
macro_rules! impl_T_x_FIELD {
    ($opr:ident $f:ident $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $opr<$field_type> for $name {
            fn $f(&mut self, rhs:$field_type) {
                self.$field.$f(rhs)
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<$field_type> for &mut $name {
            fn $f(&mut self, rhs:$field_type) {
                self.$field.$f(rhs)
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<&$field_type> for $name {
            fn $f(&mut self, rhs:&$field_type) {
                self.$field.$f(*rhs)
            }
        }

        #[allow(clippy::needless_update)]
        impl $opr<&$field_type> for &mut $name {
            fn $f(&mut self, rhs:&$field_type) {
                self.$field.$f(*rhs)
            }
        }
    )*};
}


// =============
// === Units ===
// =============

unit!{
Pixels::pixels(f32)
}

unit!{
Radians::radians(f32)
}

unit!{
Degrees::degrees(f32)
}

impl From<i32>   for Pixels { fn from(t:i32)   -> Self { (t as f32).into() } }
impl From<&i32>  for Pixels { fn from(t:&i32)  -> Self { (*t).into() } }
impl From<&&i32> for Pixels { fn from(t:&&i32) -> Self { (*t).into() } }



// ==============
// === Traits ===
// ==============

/// Commonly used traits.
pub mod traits {
    pub use super::pixels::Into  as TRAIT_IntoPixels;
    pub use super::radians::Into as TRAIT_IntoRadians;
    pub use super::degrees::Into as TRAIT_IntoDegrees;
}

pub use traits::*;
//unsigned_unit_proxy! {
///// A type representing horizontal measurements.
/////
///// **WARNING**
///// This is currently in units that are not very well defined except that ASCII characters count as
///// 1 each. This should be fixed in the future.
//Column2::column2(Bytes)
//}