use super::ContinuousTimeAnimator;
use super::FnAnimation;



// ====================================
// === DifferentialTimeAnimatorData ===
// ====================================

struct DifferentialTimeAnimatorData {
    closure          : Box<dyn FnMut(f32)>,
    previous_time    : Option<f32>
}

impl DifferentialTimeAnimatorData {
    pub fn new<F:FnAnimation>(f:F) -> Self {
        let closure          = Box::new(f);
        let previous_time    = None;
        Self { closure,previous_time }
    }
}



// ================================
// === DifferentialTimeAnimator ===
// ================================

/// This structure runs an animation every frame with the time difference from the last frame as
/// its input.
pub struct DifferentialTimeAnimator {
    _continuous_animator: ContinuousTimeAnimator
}

impl DifferentialTimeAnimator {
    pub fn new<F:FnAnimation>(f:F) -> Self {
        let mut data             = DifferentialTimeAnimatorData::new(f);
        let _continuous_animator = ContinuousTimeAnimator::new(move |current_time| {
            if let Some(previous_time) = data.previous_time {
                let delta_time = current_time - previous_time;
                (data.closure)(delta_time);
            }
            data.previous_time = Some(current_time);
        });
        Self { _continuous_animator }
    }
}
