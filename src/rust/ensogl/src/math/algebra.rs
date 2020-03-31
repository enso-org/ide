//! This module gathers common math types which are widely used in this project.

use enso_prelude::*;

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

use nalgebra;



// ==============
// === Traits ===
// ==============

/// Describes types that have a zero value.
pub trait Zero {
    fn zero() -> Self;
}


// === Impls ===

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




// ============
// === Dim1 ===
// ============

pub trait HasComponents {
    type Component;
}

/// Describes types that have the first dimension component.
pub trait Dim1 : HasComponents {
    fn x(&self) -> Self::Component;
}

/// Describes types that have the second dimension component.
pub trait Dim2 : Dim1 {
    fn y(&self) -> Self::Component;
}

/// Describes types that have the third dimension component.
pub trait Dim3 : Dim2 {
    fn z(&self) -> Self::Component;
}



// ===========
// === Abs ===
// ===========

/// Describes types that have an absolute value.
pub trait Abs {
    fn abs(&self) -> Self;
}

impl Abs for usize {
    fn abs(&self) -> Self { *self }
}


// === Impls ===

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



// =================
// === Magnitude ===
// =================

pub trait Magnitude {
    fn magnitude(&self) -> f32;
}


// === Impls ===

impl Magnitude for f32 {
    fn magnitude(&self) -> f32 {
        self.abs()
    }
}



// =================
// === Normalize ===
// =================

pub trait Normalize {
    fn normalize(&self) -> Self;
}


// === Impls ===

impl Normalize for f32 {
    fn normalize(&self) -> f32 {
        self.signum()
    }
}



// =============
// === Point ===
// =============

#[derive(Clone,Copy,Debug,Neg,Sub,Add,Div,AddAssign,From,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Point3 {
    pub vec : Vector3<f32>
}

impl Point3 {
    pub fn new(x:f32, y:f32, z:f32) -> Self {
        let vec = Vector3::new(x,y,z);
        Self {vec}
    }
}

impl Default for Point3 {
    fn default() -> Self {
        let vec = nalgebra::zero();
        Self {vec}
    }
}

impl Magnitude for Point3 {
    fn magnitude(&self) -> f32 {
        self.vec.magnitude()
    }
}

impl Normalize for Point3 {
    fn normalize(&self) -> Self {
        Self {vec:self.vec.normalize()}
    }
}

impl Mul<f32> for Point3 {
    type Output = Point3;
    fn mul(self, rhs:f32) -> Self::Output {
        let vec = self.vec * rhs;
        Self {vec}
    }
}

impl Into<Vector3<f32>> for Point3 {
    fn into(self) -> Vector3<f32> {
        self.vec
    }
}
