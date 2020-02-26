//! Root module for metric abstractions.

use super::var::*;

use crate::math::topology::unit;

pub use crate::math::topology::unit::DistanceIn;
pub use crate::math::topology::unit::AngleIn;
pub use crate::math::topology::unit::Unit;
pub use crate::math::topology::unit::Anything;
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

/// Adds `px` method to every unit that can be converted to pixel distance.
pub trait PixelDistance {
    /// Distance in pixels.
    fn px(&self) -> Var<DistanceIn<Pixels>>;
}

impl<T: unit::PixelDistance> PixelDistance for T {
    fn px(&self) -> Var<DistanceIn<Pixels>> {
        unit::PixelDistance::px(self).into()
    }
}
