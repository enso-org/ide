//! This module defines an abstraction for all types which can be used as GLSL code values.

use crate::prelude::*;



// ================
// === GlslItem ===
// ================

/// Trait describing all types which can be converted to GLSL expressions.
///
/// `GlslItem<T>` is implemented for both `T` as well as for all kind of string inputs. This allows
/// for dirty injection of GLSL code easily. For example, when moving a shape, you can write
/// `s1.translate("a","b")`, where `a` and `b` refer to variables defined in the GLSL shader. Such
/// operation is not checked during compilation, so be careful when using it, please.

pub trait GlslItem<T> {
    /// Checks if the value is zero.
    fn is_zero (&self) -> bool;

    /// Converts the value to GLSL code.
    fn to_glsl (&self) -> String;
}


// === Instances ===

impl<T> GlslItem<T> for String {
    fn is_zero (&self) -> bool   { self == "0" || self == "0.0" }
    fn to_glsl (&self) -> String { self.into() }
}

impl<T> GlslItem<T> for &String {
    fn is_zero (&self) -> bool   { *self == "0" || *self == "0.0" }
    fn to_glsl (&self) -> String { (*self).into() }
}

impl<T> GlslItem<T> for str {
    fn is_zero (&self) -> bool   { self == "0" || self == "0.0" }
    fn to_glsl (&self) -> String { self.into() }
}

impl<T> GlslItem<T> for &str {
    fn is_zero (&self) -> bool   { *self == "0" || *self == "0.0" }
    fn to_glsl (&self) -> String { (*self).into() }
}

impl GlslItem<f32> for f32 {
    fn is_zero (&self) -> bool   { *self == 0.0 }
    fn to_glsl (&self) -> String {
        let is_int = self.fract() == 0.0;
        if is_int { iformat!("{self}.0") }
        else      { iformat!("{self}")   }
    }
}
