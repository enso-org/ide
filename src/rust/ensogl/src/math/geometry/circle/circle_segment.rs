//! Provides functionality related to circle segments.
use crate::math::topology::unit::{Angle, Radians};



// =====================
// === CircleSegment ===
// =====================

/// Implements computations related to circle segments.
/// For details and background on the formulas used, see
/// https://en.wikipedia.org/wiki/Circular_segment
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct CircleSegment{
    radius: f32,
    angle : Angle<Radians>,
}

impl CircleSegment{

    /// Constructor.
    pub fn new(radius:f32, angle:Angle<Radians>) -> Self{
        CircleSegment{radius, angle}
    }

    /// The length of the direct line between the segment end points.
    pub fn chord_length(self) -> f32{
        let r     = self.radius;
        let theta = self.angle.value;

        2.0 * r * (theta * 0.5).sin()
    }

    /// The arc length of the circle segment.
    pub fn arc_length(self) -> f32{
        let r     = self.radius;
        let theta = self.angle.value;

        r * theta
    }

    /// The sagitta (height) of the segment.
    pub fn sagitta(self) -> f32{
        let r     = self.radius;
        let theta = self.angle.value;

        r * (1.0 - (theta * 0.5).cos())
    }

    /// The apothem (height) of the triangular portion.
    pub fn apothem(self) -> f32{
        self.radius - self.sagitta()
    }

}
