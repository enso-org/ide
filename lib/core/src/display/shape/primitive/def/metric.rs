//! Root module for metric abstractions.

use super::var::*;

use crate::math::topology::metric;

pub use crate::math::topology::metric::DistanceIn;
pub use crate::math::topology::metric::AngleIn;
pub use crate::math::topology::metric::Value;
pub use crate::math::topology::metric::Unknown;
pub use crate::math::topology::metric::Pixels;
pub use crate::math::topology::metric::Radians;
pub use crate::math::topology::metric::Degrees;

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

impl<T:metric::PixelDistance> PixelDistance for T {
    fn px(&self) -> Var<DistanceIn<Pixels>> {
        metric::PixelDistance::px(self).into()
    }
}
