//! This file provides the implementation of EasingAnimator.

use super::ContinuousAnimator;
use crate::animation::position::HasPosition;
use crate::animation::easing::FnEasing;
use crate::control::event_loop::EventLoop;
use crate::math::utils::linear_interpolation;

use nalgebra::Vector3;
use nalgebra::clamp;

// ======================
// === EasingAnimator ===
// ======================

/// This struct animates from `origin_position` to `target_position` using easing functions.
pub struct EasingAnimator {
    _continuous_animator : ContinuousAnimator
}

impl EasingAnimator {
    /// Creates an EasingAnimator using a `easing_function` to move `object` from
    /// `origin_position` to `target_position` in `duration_seconds`.
    pub fn new<T,F>
    (mut event_loop   : &mut EventLoop
     , easing_function  : F
     , mut object       : T
     , origin_position  : Vector3<f32>
     , target_position  : Vector3<f32>
     , duration_seconds : f64
    ) -> Self
        where T : HasPosition + 'static, F : FnEasing {
        let _continuous_animator = ContinuousAnimator::new(&mut event_loop, move |time_ms| {
            let time_seconds = time_ms as f64 / 1000.0 / duration_seconds;
            let time_seconds = clamp(time_seconds, 0.0, 1.0);
            let time_seconds = easing_function(time_seconds) as f32;
            let position = linear_interpolation(origin_position, target_position, time_seconds);
            object.set_position(position);
        });
        Self { _continuous_animator }
    }
}
