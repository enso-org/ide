//! This module gathers common math types which are widely used in this project.

#![allow(non_snake_case)]


pub use nalgebra::Matrix2;
pub use nalgebra::Matrix3;
pub use nalgebra::Matrix4;

pub use nalgebra::Matrix2x3;
pub use nalgebra::Matrix2x4;
pub use nalgebra::Matrix3x2;
pub use nalgebra::Matrix3x4;
pub use nalgebra::Matrix4x2;
pub use nalgebra::Matrix4x3;
pub use nalgebra::MatrixMN;

use nalgebra;
use nalgebra::Scalar;
use nalgebra::Matrix;
use nalgebra::ComplexField;
use nalgebra::Dim;
use nalgebra::storage::Storage;

use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;



// ==========================
// === Smart Constructors ===
// ==========================

pub type Vector2<T=f32> = nalgebra::Vector2<T>;
pub type Vector3<T=f32> = nalgebra::Vector3<T>;
pub type Vector4<T=f32> = nalgebra::Vector4<T>;

pub fn Vector2<T:Scalar>(t1:T,t2:T)           -> Vector2<T> { Vector2::new(t1,t2) }
pub fn Vector3<T:Scalar>(t1:T,t2:T,t3:T)      -> Vector3<T> { Vector3::new(t1,t2,t3) }
pub fn Vector4<T:Scalar>(t1:T,t2:T,t3:T,t4:T) -> Vector4<T> { Vector4::new(t1,t2,t3,t4) }



// ==============
// === Traits ===
// ==============

/// Describes types that have a zero value.
pub trait Zero {
    /// A zero value of this type.
    fn zero() -> Self;
}

/// Smart constructor for the `Zero` trait.
pub fn zero<T:Zero>() -> T {
    <T as Zero>::zero()
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
        impl<T:Scalar> Zero for $ty<T>
        where $ty<T> : num_traits::Zero {
            fn zero() -> Self {
                nalgebra::zero()
            }
        }
    )*};
}

gen_zero!([f32,f64] = 0.0);
gen_zero!([i32,i64,usize] = 0);
gen_zero_nalgebra!([Vector2,Vector3,Vector4,Matrix2,Matrix3,Matrix4,Matrix2x3,Matrix2x4,Matrix3x2
                   ,Matrix3x4,Matrix4x2,Matrix4x3]);



// =====================
// === HasComponents ===
// =====================

/// Every type which has components, like `Vector<f32>`.
pub trait HasComponents {
    /// The component type.
    type Component;
}


// ============
// === Dim1 ===
// ============

/// Describes types that have the first dimension component.
pub trait Dim1 : HasComponents {
    /// X-axis component getter.
    fn x(&self) -> Self::Component;
}

/// Describes types that have the second dimension component.
pub trait Dim2 : Dim1 {
    /// Y-axis component getter.
    fn y(&self) -> Self::Component;
}

/// Describes types that have the third dimension component.
pub trait Dim3 : Dim2 {
    /// Z-axis component getter.
    fn z(&self) -> Self::Component;
}



// ===========
// === Abs ===
// ===========

/// Types which have an absolute value.
pub trait Abs {
    /// Absolute value.
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



// ===========
// === Pow ===
// ===========

/// Types which can be raised to the given power.
#[allow(missing_docs)]
pub trait Pow<T=Self> {
    type Output;
    fn pow(self, t:T) -> Self::Output;
}

impl Pow<f32> for f32 {
    type Output = f32;
    fn pow(self, t:f32) -> Self::Output {
        self.powf(t)
    }
}




// =================
// === Magnitude ===
// =================

/// Types which have magnitude value.
#[allow(missing_docs)]
pub trait Magnitude {
    type Output;
    fn magnitude(&self) -> Self::Output;
}


// === Impls ===

impl Magnitude for f32 {
    type Output = f32;
    fn magnitude(&self) -> Self::Output {
        self.abs()
    }
}

impl<N:ComplexField, R:Dim, C:Dim, S:Storage<N,R,C>> Magnitude for Matrix<N,R,C,S> {
    type Output = N::RealField;
    fn magnitude(&self) -> Self::Output {
        self.norm()
    }
}



// ==============
// === Signum ===
// ==============

/// Computes the signum of the value. Returns +1 if its positive, -1 if its negative, 0 if its zero.
/// It can also return other values for specific types like `NaN` for `NaN`.
#[allow(missing_docs)]
pub trait Signum {
    type Output;
    fn signum(self) -> Self::Output;
}


// === Impls ===

impl Signum for f32 {
    type Output = f32;
    fn signum(self) -> f32 {
        f32::signum(self)
    }
}



// =============
// === Clamp ===
// =============

/// Clamps the value to [min..max] range.
#[allow(missing_docs)]
pub trait Clamp {
    type Output;
    fn clamp(self, min:Self, max:Self) -> Self::Output;
}


// === Impls ===

impl Clamp for f32 {
    type Output = f32;
    fn clamp(self, min:f32, max:f32) -> f32 {
        self.clamp(min,max)
    }
}



// ===========
// === Min ===
// ===========

#[allow(missing_docs)]
pub trait Min {
    fn min(self, other:Self) -> Self;
}


// === Impls ===

impl Min for f32 {
    fn min(self, other:Self) -> Self {
        self.min(other)
    }
}



// ===========
// === Max ===
// ===========

#[allow(missing_docs)]
pub trait Max {
    fn max(self, other:Self) -> Self;
}


// === Impls ===

impl Max for f32 {
    fn max(self, other:Self) -> Self {
        self.max(other)
    }
}



// =================
// === Normalize ===
// =================

/// Types which can be normalized.
#[allow(missing_docs)]
pub trait Normalize {
    fn normalize(&self) -> Self;
}


// === Impls ===

impl Normalize for f32 {
    fn normalize(&self) -> f32 {
        self.signum()
    }
}

impl Normalize for Vector2<f32> {
    fn normalize(&self) -> Self {
        self.normalize()
    }
}

impl Normalize for Vector3<f32> {
    fn normalize(&self) -> Self {
        self.normalize()
    }
}



// ===================
// === Square Root ===
// ===================

/// Types from which a square root can be calculated.
#[allow(missing_docs)]
pub trait Sqrt {
    type Output;
    fn sqrt(&self) -> Self::Output;
}


// === Impls ===

impl Sqrt for f32 {
    type Output = f32;
    fn sqrt(&self) -> f32 {
        f32::sqrt(*self)
    }
}



// ===========
// === Cos ===
// ===========

/// Types from which a cosine can be calculated.
#[allow(missing_docs)]
pub trait Cos {
    type Output;
    fn cos(&self) -> Self;
}


// === Impls ===

impl Cos for f32 {
    type Output = f32;
    fn cos(&self) -> f32 {
        f32::cos(*self)
    }
}



// ===========
// === Sin ===
// ===========

/// Types from which a sine can be calculated
#[allow(missing_docs)]
pub trait Sin {
    type Output;
    fn sin(&self) -> Self::Output;
}


// === Impls ===

impl Sin for f32 {
    type Output = f32;
    fn sin(&self) -> f32 {
        f32::sin(*self)
    }
}



// ============
// === Asin ===
// ============

/// Types from which a asin can be calculated
#[allow(missing_docs)]
pub trait Asin {
    type Output;
    fn asin(&self) -> Self::Output;
}


// === Impls ===

impl Asin for f32 {
    type Output = f32;
    fn asin(&self) -> f32 {
        f32::asin(*self)
    }
}



// ============
// === Acos ===
// ============

/// Types from which a asin can be calculated
#[allow(missing_docs)]
pub trait Acos {
    type Output;
    fn acos(&self) -> Self::Output;
}


// === Impls ===

impl Acos for f32 {
    type Output = f32;
    fn acos(&self) -> f32 {
        f32::acos(*self)
    }
}



// ============================
// === Algebraic Structures ===
// ============================
// TODO evaluate for correctness and usefulness.

/// Trait that describes a set of numbers that define addition, subtraction, multiplication,
/// and division.
pub trait Field<T> = Add<T,Output=T> + Sub<T,Output=T> + Mul<T,Output=T> + Div<T,Output=T>;
