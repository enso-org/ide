//! This module gathers common math types which are widely used in this project.

pub use nalgebra::Vector2;
pub use nalgebra::Vector3;
pub use nalgebra::Vector4;

pub use nalgebra::Matrix2;
pub use nalgebra::Matrix3;
pub use nalgebra::Matrix4;

pub use nalgebra::Matrix2x3;
pub use nalgebra::Matrix2x4;
pub use nalgebra::Matrix3x2;
pub use nalgebra::Matrix3x4;
pub use nalgebra::Matrix4x2;
pub use nalgebra::Matrix4x3;



// ==============
// === Traits ===
// ==============

pub trait Zero {
    fn zero() -> Self;
}

pub trait Abs {
    fn abs(&self) -> Self;
}


// === Zero ===

macro_rules! gen_zero {
    ([$($ty:ident),*] = $value:expr) => {$(
        impl Zero for $ty {
            fn zero() -> Self {
                $value
            }
        }
    )*};
}

macro_rules! gen_zero_nalgebra {
    ([$($ty:ident),*]) => {$(
        impl<T:nalgebra::Scalar+num_traits::Zero> Zero for $ty<T> {
            fn zero() -> Self {
                nalgebra::zero()
            }
        }
    )*};
}

gen_zero!([f32,f64] = 0.0);
gen_zero!([i32,i64,usize] = 0);
gen_zero_nalgebra!([Vector2,Vector3,Vector4,Matrix2,Matrix3,Matrix4,Matrix2x3,Matrix2x4,Matrix3x2,Matrix3x4,Matrix4x2,Matrix4x3]);


// === Abs ===

impl Abs for usize {
    fn abs(&self) -> Self { *self }
}

macro_rules! gen_abs {
    ([$($ty:ident),*]) => {$(
        impl Abs for $ty {
            fn abs(&self) -> Self {
                if *self < Self::zero() { -self } else { *self }
            }
        }
    )*};
}

gen_abs!([f32,f64,i32,i64]);
