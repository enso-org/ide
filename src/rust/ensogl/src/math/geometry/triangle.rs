//! This module defines an computations related to triangle geometry.

use crate::math::algebra::{Acos,Sin,Cos,Sqrt};
use crate::math::algebra::Field;

use core::f32::consts::PI;

/// Represents a triangle through its angles and side lengths.
///
/// Triangle Schematic
///----------------------
///
///                           C
///                          / \
///                        /    -\
///                      -/ gamma --\
///                   --/            --\
///               b -/                  --\ a
///              --/                       --\
///            -/                             --\
///          -/                                  --\
///       --/                                       --\
///     -/  alpha                                   beta-\
///   -/--------------------------------------------------- B
///  A                              c
///
/// Where a, b and c are the length of the respective side.
/// Where alpha, beta and gamma are the respective angle.
///
#[allow(missing_docs)]
#[derive(Debug)]
pub struct Triangle<S> {
    pub side_length_a   : S,
    pub side_length_b   : S,
    pub side_length_c   : S,
    pub angle_alpha_rad : S,
    pub angle_beta_rad  : S,
    pub angle_gamma_rad : S,
    // Prevent instantiation as something other than a result type from this module.
    dummy: (),
}

pub trait FloatLike<S> = Field<S> + Sin<Output=S> + Acos<Output=S> + Cos<Output=S> + Clone + Sqrt<Output=S> + From<f32>;

impl<S> Triangle<S>
where S:FloatLike<S> {
    /// Compute a triangle from two sides and the included angle (SAS).
    /// See https://en.wikipedia.org/wiki/Solution_of_triangles#Two_sides_and_the_included_angle_given_(SAS)
    pub fn from_sides_and_angle(side_length_a:S, side_length_b:S, angle_gamma_rad:S) -> Triangle<S> {
        let a_squared = side_length_a.clone() * side_length_a.clone();
        let b_squared = side_length_b.clone() * side_length_b.clone();

        let two = S::from(2.0_f32);

        let c_squared     = a_squared.clone() + b_squared.clone()
                            - (two.clone() * side_length_a.clone() * side_length_b.clone() * angle_gamma_rad.cos());
        let side_length_c = c_squared.sqrt();

        let angle_alpha_rad_cos = (b_squared + c_squared - a_squared)
                                  / (two * side_length_b.clone() * side_length_c.clone());
        let angle_alpha_rad     = angle_alpha_rad_cos.acos();

        let angle_beta_rad = S::from(PI) - angle_alpha_rad.clone() - angle_gamma_rad.clone();

        Triangle{
            side_length_a,
            side_length_b,
            side_length_c,
            angle_alpha_rad,
            angle_beta_rad,
            angle_gamma_rad,
            dummy: ()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::*;

    #[test]
    fn test_from_sides_and_angle() {
        let result = Triangle::from_sides_and_angle(1.0, 1.0 , 60.0_f32.to_radians());

        assert_approx_eq!(result.side_length_a, 1.0);
        assert_approx_eq!(result.side_length_b, 1.0);
        assert_approx_eq!(result.angle_gamma_rad, 60_f32.to_radians());

        assert_approx_eq!(result.side_length_c, 1.0);
        assert_approx_eq!(result.angle_beta_rad, 60.0_f32.to_radians());
        assert_approx_eq!(result.angle_alpha_rad, 60.0_f32.to_radians());

        let result = Triangle::from_sides_and_angle(1.0, 1.0 , 90.0_f32.to_radians());

        assert_approx_eq!(result.side_length_a, 1.0);
        assert_approx_eq!(result.side_length_b, 1.0);
        assert_approx_eq!(result.angle_gamma_rad, 90_f32.to_radians());

        assert_approx_eq!(result.side_length_c, 1.4142135);
        assert_approx_eq!(result.angle_beta_rad, 45.0_f32.to_radians());
        assert_approx_eq!(result.angle_alpha_rad, 45.0_f32.to_radians());

        let result = Triangle::from_sides_and_angle(1.0, 4.0 , 128.0_f32.to_radians());

        assert_approx_eq!(result.side_length_a, 1.0);
        assert_approx_eq!(result.side_length_b, 4.0);
        assert_approx_eq!(result.angle_gamma_rad, 128_f32.to_radians());

        assert_approx_eq!(result.side_length_c, 4.682445);
        assert_approx_eq!(result.angle_beta_rad, 0.7384765);
        assert_approx_eq!(result.angle_alpha_rad, 0.16909483);
    }
}
