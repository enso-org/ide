//! Definition of style sheet values.

use crate::prelude::*;



// ============
// === Data ===
// ============

/// Type of values in the style sheet.
#[derive(Debug,Clone,PartialEq)]
#[allow(missing_docs)]
pub enum Data {
    Invalid(String),
    Number(f32)
}

/// Smart constructor for `Data`.
pub fn data<T:Into<Data>>(t:T) -> Data {
    t.into()
}

impl From<f32> for Data {
    fn from(t:f32) -> Data {
        Data::Number(t)
    }
}

impl Mul<&Data> for &Data {
    type Output = Data;
    fn mul(self, rhs:&Data) -> Self::Output {
        match(self,rhs) {
            (Data::Invalid(t),_) => Data::Invalid(t.clone()),
            (_,Data::Invalid(t)) => Data::Invalid(t.clone()),
            (Data::Number(lhs),Data::Number(rhs)) => Data::Number(lhs*rhs),
            // _ => Data::Invalid("Cannot multiply.".into())
        }
    }
}

impl Add<&Data> for &Data {
    type Output = Data;
    fn add(self, rhs:&Data) -> Self::Output {
        match(self,rhs) {
            (Data::Invalid(t),_) => Data::Invalid(t.clone()),
            (_,Data::Invalid(t)) => Data::Invalid(t.clone()),
            (Data::Number(lhs),Data::Number(rhs)) => Data::Number(lhs+rhs),
            // _ => Data::Invalid("Cannot multiply.".into())
        }
    }
}

impl Eq for Data {}
