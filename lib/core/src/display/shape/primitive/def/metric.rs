//! Root module for metric abstractions.

use super::var::*;

use crate::math::topology::unit;

pub use crate::math::topology::unit::Distance;
pub use crate::math::topology::unit::Angle;
pub use crate::math::topology::unit::Unit;
pub use crate::math::topology::unit::Pixels;
pub use crate::math::topology::unit::Radians;
pub use crate::math::topology::unit::Degrees;

/// Exports common traits from this module and its sub-modules.
pub mod traits {
    pub use super::PixelDistance;
}



// =====================
// === PixelDistance ===
// =====================

/// Provides a `px` method to every unit that can be converted to a pixel distance.
pub trait PixelDistance {
    /// Distance in pixels.
    fn px(&self) -> Var<Distance<Pixels>>;
}

impl<T: unit::PixelDistance> PixelDistance for T {
    fn px(&self) -> Var<Distance<Pixels>> {
        unit::PixelDistance::px(self).into()
    }
}
