#![allow(missing_docs)]

use crate::prelude::*;
use std::ops::AddAssign; // FIXME

use ensogl::math::topology::unit::Unit;

pub mod traits {
    pub use super::ConversionToBytes as TRAIT_ConversionToBytes;
}


// ==============
// === Macros ===
// ==============

macro_rules! num_newtype {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $(#$meta)*
        #[derive(Clone,Copy,Debug,Default,Eq,From,Hash,Ord,PartialEq,PartialOrd)]
        pub struct $name { $(pub $field : $field_type),* }

        /// Smart constructor.
        $(#$meta)*
        #[allow(non_snake_case)]
        pub fn $name($($field:$field_type),*) -> $name { $name {$($field),*} }

        impl From<&$name> for $name       { fn from(t:&$name) -> Self { *t } }
        $(
        impl From<$name>  for $field_type { fn from(t:$name)  -> Self { t.$field } }
        impl From<&$name> for $field_type { fn from(t:&$name) -> Self { t.$field } }
        )*

        impl AddAssign<$name> for $name {
            fn add_assign(&mut self, rhs:Self) {
                *self = Self { $($field:self.$field.add(rhs.$field)),* }
            }
        }

        num_newtype_opr! {Sub           sub            $name {$($field:$field_type),*}}
        num_newtype_opr! {Add           add            $name {$($field:$field_type),*}}
        num_newtype_opr! {SaturatingAdd saturating_add $name {$($field:$field_type),*}}
    };
}

macro_rules! num_newtype_opr {
    ($opr:ident $f:ident $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        impl $opr<$name> for $name {
            type Output = $name;
            fn $f(self, rhs:$name) -> Self::Output {
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

        $(
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
        )*
    };
}



// ================
// === Location ===
// ================

num_newtype! {
/// An offset in the buffer in bytes.
Bytes { value : usize }
}

num_newtype! {
/// A type representing vertical measurements.
Line { raw : usize }
}

num_newtype! {
/// A type representing horizontal measurements.
///
/// **WARNING**
/// This is currently in units that are not very well defined except that ASCII characters count as
/// 1 each. This should be fixed in the future.
Column { raw : usize }
}

num_newtype! {
/// A type representing 2d measurements.
Location {
    line   : Line,
    column : Column,
}}

impl Bytes {
    pub fn as_line(self) -> Line {
        Line(self.value)
    }

    pub fn as_column(self) -> Column {
        Column(self.value)
    }
}

pub trait ConversionToBytes {
    type Output;
    fn bytes(self) -> Self::Output;
}

impl<T:Into<Bytes>> ConversionToBytes for T {
    type Output = Bytes;
    fn bytes(self) -> Self::Output {
        self.into()
    }
}

impl<T:Into<Bytes>> ConversionToBytes for Range<T> {
    type Output = Range<Bytes>;
    fn bytes(self) -> Self::Output {
        let start = self.start.bytes();
        let end   = self.end.bytes();
        Range {start,end}
    }
}

// TODO: Uncomment after updating rutc.
// impl From<Range<Bytes>> for rope::Interval {
//     fn from(t:Range<Bytes>) -> Self {
//         (t.start.raw .. t.end.raw).into()
//     }
// }


impl AddAssign<usize> for Bytes {
    fn add_assign(&mut self, rhs:usize) {
        self.value += rhs;
    }
}

//type Location = Unit<(Line,Column)>