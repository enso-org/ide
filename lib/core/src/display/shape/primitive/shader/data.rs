//! This module defines an abstraction for all types which can be used as GLSL code values.

use crate::prelude::*;

use crate::system::gpu::shader::glsl::Glsl;
use crate::system::gpu::types::*;

use nalgebra::Scalar;

use crate::display::shape::primitive::sdf::{Value,Unknown};


// ==================
// === ShaderData ===
// ==================

/// An overlapping marker trait describing all types which can be converted to GLSL expressions.
///
/// If an input is typed as `ShaderData<T>`, it accepts either `T` or any kind of string. This
/// allows for dirty injection of GLSL code easily. For example, when moving a shape, you can write
/// `s1.translate("a","b")`, where `a` and `b` refer to variables defined in the GLSL shader. Such
/// operation is not checked during compilation, so use it only when really needed.
pub trait ShaderData<T:?Sized> = ShaderDataMarker<T> + Into<Glsl>;

pub trait ShaderDataMarker<T:?Sized> {}


// === Instances ===

impl<T> ShaderDataMarker<T> for Glsl    {}
impl<T> ShaderDataMarker<T> for &Glsl   {}
impl<T> ShaderDataMarker<T> for String  {}
impl<T> ShaderDataMarker<T> for &String {}
impl<T> ShaderDataMarker<T> for &str    {}
impl<T> ShaderDataMarker<T> for  T where T:Into<Glsl> {}
impl<T> ShaderDataMarker<T> for &T where for <'t> &'t T:Into<Glsl> {}

impl<T,U,V> ShaderDataMarker<Value<T,Unknown,V>> for Value<T,U,V> where {}

impl<T,S1,S2> ShaderDataMarker<Vector2<T>> for (S1,S2)
    where T:Scalar, S1:ShaderDataMarker<T>, S2:ShaderDataMarker<T> {}



// === Any ===

/// A special version which allows for any input type.

impl<T> ShaderDataMarker<dyn Any> for  T where  T:Into<Glsl> {}
impl<T> ShaderDataMarker<dyn Any> for &T where for <'t> &'t T:Into<Glsl> {}
