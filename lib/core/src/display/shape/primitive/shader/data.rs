//! This module defines an abstraction for all types which can be used as GLSL code values.

use crate::prelude::*;

use crate::system::gpu::data::BufferItem;
use crate::system::gpu::data::ShaderDefault;
use crate::display::render::webgl::glsl::Glsl;



// ==================
// === ShaderData ===
// ==================

pub trait T1 = Sized where Glsl:From<Self>;

/// Trait describing all types which can be converted to GLSL expressions.
///
/// `ShaderData<T>` is implemented for both `T` as well as for all kind of string inputs. This
/// allows for dirty injection of GLSL code easily. For example, when moving a shape, you can write
/// `s1.translate("a","b")`, where `a` and `b` refer to variables defined in the GLSL shader. Such
/// operation is not checked during compilation, so be careful when using it, please.

pub trait ShaderData<T>: T1 {
    /// Checks if the value is zero.
    fn is_zero (&self) -> bool;

//    /// Converts the value to GLSL code.
//    fn to_glsl (&self) -> Glsl;
}


// === Instances ===

impl<T> ShaderData<T> for Glsl {
    fn is_zero (&self) -> bool { self.str == "0" || self.str == "0.0" }
//    fn to_glsl (&self) -> Glsl { self.clone() }
}

impl<T> ShaderData<T> for &Glsl {
    fn is_zero (&self) -> bool { (*self).str == "0" || (*self).str == "0.0" }
//    fn to_glsl (&self) -> Glsl { (*self).clone() }
}

impl<T> ShaderData<T> for String {
    fn is_zero (&self) -> bool { self == "0" || self == "0.0" }
//    fn to_glsl (&self) -> Glsl { Glsl{str:self.into()} }
}

impl<T> ShaderData<T> for &String {
    fn is_zero (&self) -> bool { *self == "0" || *self == "0.0" }
//    fn to_glsl (&self) -> Glsl { Glsl{str:(*self).into()} }
}

//impl<T> ShaderData<T> for str {
//    fn is_zero (&self) -> bool { self == "0" || self == "0.0" }
////    fn to_glsl (&self) -> Glsl { Glsl{str:self.into()} }
//}

impl<T> ShaderData<T> for &str {
    fn is_zero (&self) -> bool { *self == "0" || *self == "0.0" }
//    fn to_glsl (&self) -> Glsl { Glsl{str:(*self).into()} }
}

impl<T: BufferItem+PartialEq+T1> ShaderData<T> for T {
    fn is_zero (&self) -> bool { <T as ShaderDefault> :: is_shader_default(self) }
//    fn to_glsl (&self) -> Glsl { Glsl{str:<T as BufferItem>    :: to_glsl(self)}  }
}
