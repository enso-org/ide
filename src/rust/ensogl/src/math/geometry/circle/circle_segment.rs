//! Provides functionality related to circle segments.
use std::ops::*;
use crate::math::{Cos, Sin};


// =====================
// === CircleSegment ===
// =====================

/// Implements computations related to circle segments.
/// For details and background on the formulas used, see
/// https://en.wikipedia.org/wiki/Circular_segment
#[derive(Clone,Debug,PartialEq)]
pub struct CircleSegment<S>{
    radius: S,
    angle : S,
}

impl<S> CircleSegment<S>
where S: Add<S,Output=S> + Mul<S,Output=S> + Sub<S,Output=S> + Sin + Cos + From<f32> + Clone{

    /// Constructor.
    pub fn new(radius:S, angle:S) -> Self{
        CircleSegment{radius, angle}
    }

    /// The length of the direct line between the segment end points.
    pub fn chord_length(&self) -> S{
        let radius = self.radius.clone();
        let theta  = self.angle.clone();

       S::from(2.0_f32) * radius * (theta * S::from(0.5_f32)).sin()
    }

    /// The arc length of the circle segment.
    pub fn arc_length(&self) -> S{
        let radius = self.radius.clone();
        let theta  = self.angle.clone();

        radius * theta
    }

    /// The sagitta (height) of the segment.
    pub fn sagitta(&self) -> S{
        let radius = self.radius.clone();
        let theta  = self.angle.clone();

        radius * (S::from(1.0_f32) - (theta * S::from(0.5_f32)).cos())
    }

    /// The apothem (height) of the triangular portion.
    pub fn apothem(self) -> S{
        self.radius.clone() - self.sagitta()
    }

}
