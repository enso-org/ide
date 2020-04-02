use core::f32::consts::PI;
use crate::display::shape::Var;
use crate::math::algebra::{Sin, Cos, Sqrt};
use crate::math::topology::unit::{Radians, Angle};
use std::ops::*;

pub struct Triangle<S> {
    pub a     : S,
    pub b     : S,
    pub c     : S,
    pub alpha : S,
    pub beta  : S,
    pub gamma : S,
}

impl<S> Triangle<S>
    where S: Sub<S,Output=S> + Add<S,Output=S> + Mul<S,Output=S> +Div<S,Output=S> + Sin + Cos + Clone + Sqrt + From<f32> {
    /// Two sides and the included angle given (SAS)
    /// TODO[mm]: testing
    pub fn from_sas(a:S, b:S, gamma:S) -> Triangle<S> {
        let a2 = (a.clone() * a.clone());
        let b2 = (b.clone() * b.clone());

        let c2 = a2.clone() + b2.clone() - (S::from(2.0_f32) * a.clone() * b.clone() * gamma.cos());
        let c = c2.sqrt();

        let alpha = (b2.clone() + c2.clone() - a2.clone()) / (S::from(2.0_f32) * b.clone() * c.clone());

        let beta = S::from(PI) - alpha.clone() - gamma.clone();

        Triangle{a,b,c,alpha,beta,gamma}
    }
}
