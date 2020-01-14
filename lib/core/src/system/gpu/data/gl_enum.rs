//! This module defines a wrapper for WebGL enums and associated utils.

use crate::prelude::*;
use crate::display::render::webgl::Context;



// ==============
// === GlEnum ===
// ==============

/// The newtype for WebGL enums.
#[derive(Clone,Copy,Debug)]
pub struct GlEnum {
    /// Raw value of the enum.
    pub raw: u32,
}

impl From<u32> for GlEnum {
    fn from(raw:u32) -> Self {
        Self {raw}
    }
}

impl From<GlEnum> for u32 {
    fn from(t:GlEnum) -> Self {
        t.raw
    }
}



// ==================
// === Extensions ===
// ==================

/// Extension methods.
pub mod traits {
    use super::*;

    /// Methods for every object which implements `Into<GlEnum>`.
    pub trait GlEnumOps {
        /// Converts the current value to `GlEnum`.
        fn to_gl_enum<G:From<GlEnum>>(&self) -> G;
    }

    impl<T> GlEnumOps for T where for<'a> &'a T:Into<GlEnum> {
        fn to_gl_enum<G:From<GlEnum>>(&self) -> G {
            let g:GlEnum = self.into();
            g.into()
        }
    }

    /// Methods for every object which implements `Into<GlEnum>`.
    pub trait IsGlEnum {
        /// Converts the current value to `GlEnum`.
        fn gl_enum<G:From<GlEnum>>() -> G;
    }

    impl<T> IsGlEnum for T where PhantomData<T>:Into<GlEnum> {
        fn gl_enum<G:From<GlEnum>>() -> G {
            let g:GlEnum = PhantomData::<T>.into();
            g.into()
        }
    }
}



// ==============
// === Macros ===
// ==============

/// Combination of `define_singletons` and `define_gl_enum_conversions`.
#[macro_export]
macro_rules! define_singletons_gl {
    ( $( $(#$meta:tt)* $name:ident = $expr:expr ),* $(,)? ) => {
        shapely::define_singletons!{ $( $(#$meta)* $name),* }
        $crate::define_gl_enum_conversions!{ $( $(#$meta)* $name = $expr ),* }
    }
}


/// Defines conversions `From<$type>` and `From<PhantomData<$type>>` for every provided type.
#[macro_export]
macro_rules! define_gl_enum_conversions {
    ( $( $(#$meta:tt)* $type:ty = $expr:expr ),* $(,)? ) => {
        $(
            impl From<$type> for GlEnum {
                fn from(_:$type) -> Self {
                    $expr.into()
                }
            }

            impl From<PhantomData<$type>> for GlEnum {
                fn from(_:PhantomData<$type>) -> Self {
                    $expr.into()
                }
            }
        )*
    }
}

/// Defines singletons and an associated enum type, just like `define_singleton_enum`.
/// It also defines conversions `From<$singleton>` and `From<PhantomData<$singleton>>` for every
/// singleton type and for the whole enum type.
#[macro_export]
macro_rules! define_singleton_enum_gl {
    (
        $(#$meta:tt)*
        $name:ident {
            $( $(#$field_meta:tt)* $field:ident = $expr:expr),* $(,)?
        }
    ) => {
        $crate  :: define_singletons_gl!       { $($(#$field_meta)* $field = $expr),* }
        shapely :: define_singleton_enum_from! { $(#$meta)* $name {$($(#$field_meta)* $field),*}}

        impl From<&$name> for GlEnum {
            fn from(t:&$name) -> Self {
                match t {
                    $($name::$field => $field.into()),*
                }
            }
        }
    }
}


// ================================
// === Primitive Type Instances ===
// ================================

define_gl_enum_conversions! {
    bool = Context::BOOL,
    i32  = Context::INT,
    f32  = Context::FLOAT,
}
