//! Provides functionality related to circle segments.

use crate::math::{Cos,Sin,Field};

use core::fmt::Debug;



// =====================
// === CircleSegment ===
// =====================

/// Implements computations related to circle segments. For details and background on the formulas
/// used, see https://en.wikipedia.org/wiki/Circular_segment
#[derive(Clone,Debug,PartialEq)]
pub struct CircleSegment<T> {
    radius : T,
    angle  : T,
}

pub trait FloatLike<T> = Field<T> + Sin<Output=T> + Cos<Output=T> + From<f32> + Clone + Debug;

impl<T> CircleSegment<T>
where T: FloatLike<T> {

    /// Constructor. Angle is required to be in radians.
    pub fn new(radius:T, angle:T) -> Self {
        CircleSegment{radius,angle}
    }

    /// The length of the direct line between the segment end points.
    pub fn chord_length(&self) -> T {
        let radius = self.radius.clone();
        let theta  = self.angle.clone();
        T::from(2.0_f32) * radius * (theta * T::from(0.5_f32)).sin()
    }

    /// The arc length of the circle segment.
    pub fn arc_length(&self) -> T {
        let radius = self.radius.clone();
        let theta  = self.angle.clone();
        radius * theta
    }

    /// The sagitta (height) of the segment.
    pub fn sagitta(&self) -> T {
        let radius = self.radius.clone();
        let theta  = self.angle.clone();
        radius * (T::from(1.0_f32) - (theta * T::from(0.5_f32)).cos())
    }

    /// The apothem (height) of the triangular portion.
    pub fn apothem(self) -> T {
        self.radius.clone() - self.sagitta()
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    // TODO more testing

    #[test]
    fn check_chord_Length() {
        let segment = CircleSegment::new(1.0, 1.4);
        debug_assert_ne!(segment.chord_length(), 0.0);
    }
}
