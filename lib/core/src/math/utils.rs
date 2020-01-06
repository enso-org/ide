use nalgebra::clamp;
use std::ops::Mul;
use std::ops::Add;

pub fn linear_interpolation<T>(a:T, b:T, t:f32) -> T
    where T : Mul<f32, Output = T> + Add<T, Output = T> {
    a * (1.0 - t) + b * t
}
