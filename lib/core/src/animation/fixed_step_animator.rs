use super::DifferentialTimeAnimator;
use super::FnAnimation;

use nalgebra::zero;

// =============================
// === FixedStepAnimatorData ===
// =============================

struct FixedStepAnimatorData {
    closure          : Box<dyn FnMut(f32)>,
    step_duration    : f32,
    accumulated_time : f32
}

impl FixedStepAnimatorData {
    pub fn new<F:FnAnimation>(steps_per_second:f32, f:F) -> Self {
        let closure          = Box::new(f);
        let step_duration    = 1000.0 / steps_per_second;
        let accumulated_time = zero();
        Self { closure,step_duration,accumulated_time }
    }
}



// ================
// === Animator ===
// ================

/// This structure is aimed to run a closure at a fixed time rate.
pub struct FixedStepAnimator {
    _differential_animator: DifferentialTimeAnimator
}

impl FixedStepAnimator {
    pub fn new<F:FnAnimation>(steps_per_second:f32, f:F) -> Self {
        let mut data               = FixedStepAnimatorData::new(steps_per_second, f);
        let _differential_animator = DifferentialTimeAnimator::new(move |delta_time| {
            data.accumulated_time += delta_time;
            while data.accumulated_time > data.step_duration {
                data.accumulated_time -= data.step_duration;
                (data.closure)(data.step_duration);
            }
        });
        Self { _differential_animator }
    }
}
