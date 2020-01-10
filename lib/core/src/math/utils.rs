//! This module aims to provide math utils.

pub trait Interpolatable<T> = Mul<f32, Output = T> + Add<T, Output = T>;

use std::ops::Mul;
use std::ops::Add;

/// Linear interpolation function for any type implementing T * f32 and T + T.
pub fn linear_interpolation<T:Interpolatable<T>>(a:T, b:T, t:f32) -> T {
    a * (1.0 - t) + b * t
}
