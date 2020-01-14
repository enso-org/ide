//! This module exports primitive data and associated utils.



// =============
// === Types ===
// =============

/// `Identity<A>` resolves to `A`.
pub type Identity<T> = T;



// ==============
// === Macros ===
// ==============

/// Evaluates the argument macro with a list of all Rust built-in types supported on GPU.
#[macro_export]
macro_rules! with_all_built_in_prim_types {
    ([[$f:path] $args:tt]) => {
        $f! { $args [ f32 ] }
    }
}

/// Evaluates the argument macro with a list of all nalgebra types supported on GPU.
#[macro_export]
macro_rules! with_all_nalgebra_prim_types {
    ([[$f:path] $args:tt]) => {
        $f! { $args
        [ Vector2 Vector3 Vector4 Matrix4 Matrix2 Matrix3
          Matrix2x3 Matrix2x4 Matrix3x2 Matrix3x4 Matrix4x2 Matrix4x3
        ] }
    }
}

/// Evaluates the argument macro with a list of all container types supported on GPU.
#[macro_export]
macro_rules! with_all_prim_container_types {
    ($f:tt) => {
        $crate::with_all_nalgebra_prim_types! {
            [[$crate::_with_all_prim_container_types_impl] $f]
        }
    }
}

/// Internal helper for `with_all_prim_container_types`.
#[macro_export]
macro_rules! _with_all_prim_container_types_impl {
    ([[$f:path] $args:tt] [$($types:tt)*]) => {
        $f! { $args [Identity $($types)*] }
    }
}

/// Evaluates the argument macro with a list of pairs `[container item]` for all container and for
/// all primitive types supported on GPU. One of the container type is `Identity` which just
/// resolves to it's argument.
#[macro_export]
macro_rules! with_all_prim_types {
    ([[$f:path] $args:tt]) => {
        $f! { $args
            [[Identity i32] [Identity f32] [Identity bool] [Vector2 f32] [Vector3 f32] [Vector4 f32]
             [Vector2 i32] [Vector3 i32] [Vector4 i32] [Vector2 bool] [Vector3 bool] [Vector4 bool]
             [Matrix2 f32] [Matrix3 f32] [Matrix4 f32] [Matrix2x3 f32] [Matrix2x4 f32]
             [Matrix3x2 f32] [Matrix3x4 f32] [Matrix4x2 f32] [Matrix4x3 f32]]
        }
    }
}



// ===============
// === Imports ===
// ===============

macro_rules! define_pub_use {
    ([$base:ident] [$($target:ident)*]) => {
        $(pub use $base::$target;)*
    }
}

with_all_nalgebra_prim_types!([[define_pub_use] [nalgebra]]);
