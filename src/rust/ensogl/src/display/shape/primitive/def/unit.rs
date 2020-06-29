//! Root module for metric abstractions.

use super::var::*;

use crate::math::topology::unit;

pub use crate::math::topology::unit::Unit;
pub use crate::math::topology::unit::Pixels;
pub use crate::math::topology::unit::Radians;
pub use crate::math::topology::unit::Degrees;



// =====================
// === PixelDistance ===
// =====================

/// Provides a `px` method to every unit that can be converted to a pixel distance.
#[allow(missing_docs)]
pub trait PixelDistance {
    type Output;
    fn px(&self) -> Self::Output;
}

impl PixelDistance for i32 {
    type Output = Var<Pixels>;
    fn px(&self) -> Self::Output {
        unit::ToPixels::px(self).into()
    }
}

impl PixelDistance for f32 {
    type Output = Var<Pixels>;
    fn px(&self) -> Self::Output {
        unit::ToPixels::px(self).into()
    }
}



// ===============
// === Exports ===
// ===============

/// Common types.
pub mod types {
    use super::*;
    pub use super::PixelDistance;
    pub use unit::Unit;
    pub use unit::Pixels;
    pub use unit::Radians;
    pub use unit::Degrees;
}

pub use types::*;
