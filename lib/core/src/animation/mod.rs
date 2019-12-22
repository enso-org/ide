pub mod physics;
mod continuous_time_animator;
mod differential_time_animator;
mod fixed_step_animator;

pub use continuous_time_animator::ContinuousTimeAnimator;
pub use differential_time_animator::DifferentialTimeAnimator;
pub use fixed_step_animator::FixedStepAnimator;



// ===================
// === FnAnimation ===
// ===================

pub trait FnAnimation = FnMut(f32) + 'static;



// FIXME: The objects in this section needs a better place.
// =============
// === Utils ===
// =============

use nalgebra::clamp;
use std::ops::Mul;
use std::ops::Add;

pub fn linear_interpolation<T>(a:T, b:T, t:f32) -> T
where T : Mul<f32, Output = T> + Add<T, Output = T> {
    let t = clamp(t, 0.0, 1.0);
    a * (1.0 - t) + b * t
}
