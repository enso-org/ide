//! Defines abstraction for data types that have a default value when used in shaders.

use nalgebra::*;



// =====================
// === ShaderDefault ===
// =====================

/// Trait for types which have a default value when used in shaders.
pub trait ShaderDefault {
    /// Default value for this type.
    fn shader_default() -> Self;

    /// Checks if the current value is the same as the default one.
    fn is_shader_default(&self) -> bool where Self:Sized+PartialEq {
        *self == Self::shader_default()
    }
}


// === Instances ===

macro_rules! define_shader_defaults {
    ($($ty:ty = $val:expr),* $(,)?) => {$(
        impl ShaderDefault for $ty { fn shader_default() -> Self { $val } }
    )*}
}

define_shader_defaults! {
    i32          = 0,
    f32          = 0.0,
    Vector2<f32> = Vector2::new(0.0,0.0),
    Vector3<f32> = Vector3::new(0.0,0.0,0.0),
    Vector4<f32> = Vector4::new(0.0,0.0,0.0,0.0),
    Matrix4<f32> = Matrix4::identity(),
}
