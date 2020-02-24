
use super::var::*;

use crate::math::topology::metric::DistanceOps;

pub use crate::math::topology::metric::DistanceIn;
pub use crate::math::topology::metric::AngleIn;
pub use crate::math::topology::metric::Value;
pub use crate::math::topology::metric::Unknown;
pub use crate::math::topology::metric::Pixels;
pub use crate::math::topology::metric::Radians;
pub use crate::math::topology::metric::Degrees;

pub trait PixelDistance {
    fn px(&self) -> Var<DistanceIn<Pixels>>;
}

impl<T:DistanceOps> PixelDistance for T {
    fn px(&self) -> Var<DistanceIn<Pixels>> {
        DistanceOps::px(self).into()
    }
}

pub mod traits {
    pub use super::PixelDistance;
}