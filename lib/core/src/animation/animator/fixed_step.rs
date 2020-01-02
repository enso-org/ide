use super::Animator;
use super::FnAnimation;

use nalgebra::zero;

// FIXME: Remove all unnecessary .gitkeep files, please! :) .gitkeep are used only for git not to remove a folder when no other files are present.

// =======================
// === IntervalCounter ===
// =======================

/// This struct counts the intervals in a time period.
pub struct IntervalCounter {
    pub interval_duration : f32,
    pub accumulated_time  : f32
}

impl IntervalCounter {
    pub fn new(interval_duration:f32) -> Self {
        let accumulated_time = zero();
        Self { interval_duration, accumulated_time }
    }

    /// Adds time to the counter and returns the number of intervals it reached.
    pub fn add_time(&mut self, time:f32) -> u32 {
        self.accumulated_time += time;
        let count = (self.accumulated_time / self.interval_duration) as u32;
        self.accumulated_time -= count as f32 * self.interval_duration;
        count
    }

}
// FIXME: spacing
// =============================
// === FixedStepAnimatorData ===
// =============================

struct FixedStepAnimatorData {
    closure : Box<dyn FnMut(f32)>,
    counter : IntervalCounter
}

impl FixedStepAnimatorData {
    pub fn new<F:FnAnimation>(steps_per_second:f32, f:F) -> Self {
        let closure          = Box::new(f);
        let step_duration    = 1000.0 / steps_per_second;
        let counter          = IntervalCounter::new(step_duration);
        Self { closure,counter }
    }
}



// ================
// === Animator ===
// ================

/// This structure attempts to run a closure at a fixed time rate.
///
/// # Internals
/// If, for instance, we want to run FnAnimation once per second, it's delta_time
/// (FnAnimation(delta_time)) will be 1 second. But keep in mind that if the actual frame takes
/// longer, say 2 seconds, FnAnimation will be called twice in the same moment, but its delta_time
/// parameter will always be fixed to 1 second.
pub struct FixedStepAnimator {
    _animator: Animator
}

impl FixedStepAnimator {
    pub fn new<F:FnAnimation>(steps_per_second:f32, f:F) -> Self {
        let mut data               = FixedStepAnimatorData::new(steps_per_second, f); // FIXME: Spacing
        let _animator = Animator::new(move |delta_ms| {
            let intervals = data.counter.add_time(delta_ms);
            for _ in 0..intervals {
                (data.closure)(data.counter.interval_duration);
            }
        });
        Self { _animator }
    }
}
