//! Defines utilities for creating custom strongly typed units.



// ==============
// === Macros ===
// ==============

/// Define a new unit type. Units are strongly typed wrappers for some primitive values. For
/// example, unit `Angle` could be a wrapper for `f32`.
///
/// Units automatically implement a lot of traits in a generic fashion, so you can for example add
/// to angles together, or divide angle by a number, but you are not allowed to divide a number by
/// an angle. Rarely you may want to use very custom rules for unit definition. In such a case, you
/// should use other macros defined in this module. Look at the definitions below to learn the
/// usage patterns.
///
/// ## Implementation Notes
/// You may wonder why this utility is defined as a macro that generates hundreds of lines of code
/// instead of a generic type `Unit<Phantom,Repr>`. The later approach has one major issue. The
/// definition `type Angle = Unit<AngleType,f32>` will be dysfunctional, as you would not be allowed
/// to implement many impls because the type `Unit` would be defined in an external crate.
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


/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! unsigned_unit {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub::sub for $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add::add for $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {SaturatingAdd::saturating_add for $name}
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

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! unsigned_unit_proxy {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub::sub for $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add::add for $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {SaturatingAdd::saturating_add for $name}
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

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! unsigned_unit_float_like {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct_float_like! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub::sub for $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add::add for $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {SaturatingAdd::saturating_add for $name}
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

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! signed_unit {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub::sub for $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add::add for $name}
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

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! signed_unit_float_like {
    ($(#$meta:tt)* $name:ident :: $vname:ident ($field_type:ty)) => {
        /// Unit module.
        pub mod $vname {
            use super::*;
            use std::ops::AddAssign;

            $crate::newtype_struct_float_like! {$(#$meta)* $name {value : $field_type}}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Sub::sub for $name}
            $crate::impl_UNIT_x_UNIT_to_UNIT!  {Add::add for $name}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_FIELD_to_UNIT! {Div div $name $field_type}
            $crate::impl_FIELD_x_UNIT_to_UNIT! {Mul mul $name $field_type}
            $crate::impl_UNIT_x_UNIT_to_FIELD! {Div div $name $field_type}
            $crate::impl_UNIT_x_UNIT!          {AddAssign add_assign $name}
            $crate::impl_UNIT_to_UNIT!         {Neg neg $name}

            /// Unit conversion and associated method. It has associated type in order to allow
            /// complex conversions, like `(10,10).px()` be converted the same way as
            /// `(10.px(),10.px())`.
            #[allow(missing_docs)]
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

            /// Exports. The traits are renamed not to pollute the scope.
            pub mod export {
                pub use super::$name;
                pub use super::Into as TRAIT_Into;
            }
        }
        pub use $vname::export::*;
    };
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! newtype {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        use std::ops::AddAssign;

        $crate::newtype_struct! {$(#$meta)* $name { $($field : $field_type),*}}

        $crate::impl_T_x_T_to_T! {Sub           :: sub            for $name {$($field),*}}
        $crate::impl_T_x_T_to_T! {Add           :: add            for $name {$($field),*}}
        $crate::impl_T_x_T_to_T! {SaturatingAdd :: saturating_add for $name {$($field),*}}

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

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! newtype_struct {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $crate::newtype_struct_def!   {$(#$meta)* $name { $($field : $field_type),*}}
        $crate::newtype_struct_impls! {$(#$meta)* $name { $($field : $field_type),*}}
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! newtype_struct_float_like {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $crate::newtype_struct_def_float_like! {$(#$meta)* $name { $($field : $field_type),*}}
        $crate::newtype_struct_impls!          {$(#$meta)* $name { $($field : $field_type),*}}
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! newtype_struct_def {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $(#$meta)*
        #[derive(Clone,Copy,Debug,Default,Eq,Hash,Ord,PartialEq,PartialOrd)]
        pub struct $name { $(pub $field : $field_type),* }
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! newtype_struct_def_float_like {
    ($(#$meta:tt)* $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        $(#$meta)*
        #[allow(missing_docs)]
        #[derive(Clone,Copy,Debug,Default,PartialEq,PartialOrd)]
        pub struct $name { $(pub $field : $field_type),* }
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
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

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_UNIT_x_UNIT_to_UNIT {
    ($trait:ident :: $opr:ident for $name:ident) => {
        $crate::impl_T_x_T_to_T! {$trait :: $opr for  $name {value}}
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_T_x_T_to_T {
    ($trait:ident :: $opr:ident for $name:ident { $($field:ident),* $(,)? }) => {
        impl $trait<$name> for $name {
            type Output = $name;
            fn $opr(self, rhs:$name) -> Self::Output {
                $(let $field = self.$field.$opr(rhs.$field);)*
                $name { $($field),* }
            }
        }

        impl $trait<$name> for &$name {
            type Output = $name;
            fn $opr(self, rhs:$name) -> Self::Output {
                $(let $field = self.$field.$opr(rhs.$field);)*
                $name { $($field),* }
            }
        }

        impl $trait<&$name> for $name {
            type Output = $name;
            fn $opr(self, rhs:&$name) -> Self::Output {
                $(let $field = self.$field.$opr(rhs.$field);)*
                $name { $($field),* }
            }
        }

        impl $trait<&$name> for &$name {
            type Output = $name;
            fn $opr(self, rhs:&$name) -> Self::Output {
                $(let $field = self.$field.$opr(rhs.$field);)*
                $name { $($field),* }
            }
        }
    };
}



// ======================
// === T x FIELD -> T ===
// ======================

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_UNIT_x_FIELD_to_UNIT {
    ($trait:ident $opr:ident $name:ident $field_type:ty) => {
        $crate::impl_T_x_FIELD_to_T! {$trait $opr $name {value : $field_type}}
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_T_x_FIELD_to_T {
    ($trait:ident $opr:ident $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $trait<$field_type> for $name {
            type Output = $name;
            fn $opr(self, rhs:$field_type) -> Self::Output {
                $name { $field:self.$field.$opr(rhs), ..self }
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<$field_type> for &$name {
            type Output = $name;
            fn $opr(self, rhs:$field_type) -> Self::Output {
                $name { $field:self.$field.$opr(rhs), ..*self }
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<&$field_type> for $name {
            type Output = $name;
            fn $opr(self, rhs:&$field_type) -> Self::Output {
                $name { $field:self.$field.$opr(*rhs), ..self }
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<&$field_type> for &$name {
            type Output = $name;
            fn $opr(self, rhs:&$field_type) -> Self::Output {
                $name { $field:self.$field.$opr(*rhs), ..*self }
            }
        }
    )*};
}


// ======================
// === FIELD x T -> T ===
// ======================

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_FIELD_x_UNIT_to_UNIT {
    ($trait:ident $opr:ident $name:ident $field_type:ty) => {
        $crate::impl_FIELD_x_T_to_T! {$trait $opr $name {value : $field_type}}
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_FIELD_x_T_to_T {
    ($trait:ident $opr:ident $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $trait<$name> for $field_type {
            type Output = $name;
            fn $opr(self, rhs:$name) -> Self::Output {
                $name { $field:self.$opr(rhs.$field), ..rhs }
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<$name> for &$field_type {
            type Output = $name;
            fn $opr(self, rhs:$name) -> Self::Output {
                $name { $field:self.$opr(rhs.$field), ..rhs }
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<&$name> for $field_type {
            type Output = $name;
            fn $opr(self, rhs:&$name) -> Self::Output {
                $name { $field:self.$opr(rhs.$field), ..*rhs }
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<&$name> for &$field_type {
            type Output = $name;
            fn $opr(self, rhs:&$name) -> Self::Output {
                $name { $field:self.$opr(rhs.$field), ..*rhs }
            }
        }
    )*};
}



// ======================
// === T x T -> FIELD ===
// ======================

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_UNIT_x_UNIT_to_FIELD {
    ($trait:ident $opr:ident $name:ident $field_type:ty) => {
        $crate::impl_T_x_T_to_FIELD! {$trait $opr $name {value : $field_type}}
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_T_x_T_to_FIELD {
    ($trait:ident $opr:ident $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {$(
        impl $trait<$name> for $name {
            type Output = $field_type;
            fn $opr(self, rhs:$name) -> Self::Output {
                self.$field.$opr(rhs.$field)
            }
        }

        impl $trait<$name> for &$name {
            type Output = $field_type;
            fn $opr(self, rhs:$name) -> Self::Output {
                self.$field.$opr(rhs.$field)
            }
        }

        impl $trait<&$name> for $name {
            type Output = $field_type;
            fn $opr(self, rhs:&$name) -> Self::Output {
                self.$field.$opr(rhs.$field)
            }
        }

        impl $trait<&$name> for &$name {
            type Output = $field_type;
            fn $opr(self, rhs:&$name) -> Self::Output {
                self.$field.$opr(rhs.$field)
            }
        }
    )*};
}



// ==============
// === T -> T ===
// ==============

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_UNIT_to_UNIT {
    ($trait:ident $opr:ident $name:ident) => {
        $crate::impl_T_to_T! {$trait $opr $name {value}}
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_T_to_T {
    ($trait:ident $opr:ident $name:ident { $($field:ident),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $trait for $name {
            type Output = $name;
            fn $opr(self) -> Self::Output {
                $name { $field:self.$field.$opr(), ..self }
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait for &$name {
            type Output = $name;
            fn $opr(self) -> Self::Output {
                $name { $field:self.$field.$opr(), ..*self }
            }
        }
    )*};
}




// =============
// === T x T ===
// =============

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_UNIT_x_UNIT {
    ($trait:ident $opr:ident $name:ident) => {
        $crate::impl_T_x_T! {$trait $opr $name {value}}
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_T_x_T {
    ($trait:ident $opr:ident $name:ident { $($field:ident),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $trait<$name> for $name {
            fn $opr(&mut self, rhs:$name) {
                self.$field.$opr(rhs.$field)
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<$name> for &mut $name {
            fn $opr(&mut self, rhs:$name) {
                self.$field.$opr(rhs.$field)
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<&$name> for $name {
            fn $opr(&mut self, rhs:&$name) {
                self.$field.$opr(rhs.$field)
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<&$name> for &mut $name {
            fn $opr(&mut self, rhs:&$name) {
                self.$field.$opr(rhs.$field)
            }
        }
    )*};
}



// =================
// === T x FIELD ===
// =================

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_UNIT_x_FIELD {
    ($trait:ident $opr:ident $name:ident $field_type:ty) => {
        $crate::impl_T_x_FIELD! {$trait $opr $name {value : $field_type}}
    }
}

/// Unit definition macro. See docs of `unit` to learn more.
#[macro_export]
macro_rules! impl_T_x_FIELD {
    ($trait:ident $opr:ident $name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {$(
        #[allow(clippy::needless_update)]
        impl $trait<$field_type> for $name {
            fn $opr(&mut self, rhs:$field_type) {
                self.$field.$opr(rhs)
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<$field_type> for &mut $name {
            fn $opr(&mut self, rhs:$field_type) {
                self.$field.$opr(rhs)
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<&$field_type> for $name {
            fn $opr(&mut self, rhs:&$field_type) {
                self.$field.$opr(*rhs)
            }
        }

        #[allow(clippy::needless_update)]
        impl $trait<&$field_type> for &mut $name {
            fn $opr(&mut self, rhs:&$field_type) {
                self.$field.$opr(*rhs)
            }
        }
    )*};
}
