//! Defines abstraction for data types that have a default value when used as GPU values.

use nalgebra::*;



// ==================
// === GpuDefault ===
// ==================

/// Trait for types which have a default value when used as GPU values.
pub trait GpuDefault {
    /// Default value for this type.
    fn gpu_default() -> Self;

    /// Checks if the current value is the same as the default one.
    fn is_gpu_default(&self) -> bool where Self:Sized+PartialEq {
        *self == Self::gpu_default()
    }
}


// === Instances ===

macro_rules! define_gpu_defaults {
    ($($ty:ty = $val:expr),* $(,)?) => {$(
        impl GpuDefault for $ty { fn gpu_default() -> Self { $val } }
    )*}
}

define_gpu_defaults! {
    i32          = 0,
    f32          = 0.0,
    Vector2<f32> = Vector2::new(0.0,0.0),
    Vector3<f32> = Vector3::new(0.0,0.0,0.0),
    Vector4<f32> = Vector4::new(0.0,0.0,0.0,0.0),
    Matrix4<f32> = Matrix4::identity(),
}
