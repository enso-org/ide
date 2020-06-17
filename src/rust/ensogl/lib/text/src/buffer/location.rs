
use crate::prelude::*;



// ==============
// === Macros ===
// ==============

macro_rules! num_newtype {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $(#$meta)*
        #[derive(Clone,Copy,Debug,Eq,From,Hash,Ord,PartialEq,PartialOrd)]
        pub struct $name { $(pub $field : $field_type),* }

        /// Smart constructor.
        $(#$meta)*
        pub fn $name($($field:$field_type),*) -> $name { $name {$($field),*} }

        impl From<&$name> for $name       { fn from(t:&$name) -> Self { *t } }
        $(
        impl From<$name>  for $field_type { fn from(t:$name)  -> Self { t.$field } }
        impl From<&$name> for $field_type { fn from(t:&$name) -> Self { t.$field } }
        )*

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

        $(
            impl $opr<$field_type> for $name {
                type Output = $name;
                fn $f(self, rhs:$field_type) -> Self::Output {
                    $name { $field:self.$field.$f(rhs), ..self }
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
ByteOffset { raw : usize }
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

impl ByteOffset {
    pub fn as_line(self) -> Line {
        Line(self.raw)
    }

    pub fn as_column(self) -> Column {
        Column(self.raw)
    }
}