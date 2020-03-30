use crate::math::topology::unit::{Radians, Angle};
use core::f32::consts::PI;

pub struct Triangle {
    pub a     : f32,
    pub b     : f32,
    pub c     : f32,
    pub alpha : Angle<Radians>,
    pub beta  : Angle<Radians>,
    pub gamma : Angle<Radians>,
}

impl Triangle{
    /// Two sides and the included angle given (SAS)
    pub fn from_sas(a:f32, b:f32, gamma:Angle<Radians>) -> Triangle {
        let c_squared = a.powi(2) + b.powi(2) - (2.0 * a * b * gamma.value.cos());
        let c = c_squared.sqrt();

        let alpha = (b.powi(2) + c.powi(2) - a.powi(2)) / (2.0 * b * c);
        let alpha = Angle::new(alpha);

        let beta = PI - alpha.value - gamma.value;
        let beta = Angle::new(beta);

        Triangle{a,b,c,alpha,beta,gamma}
    }
}
