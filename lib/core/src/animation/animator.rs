use crate::system::web::AnimationFrameLoop;

use nalgebra::zero;

// ===================
// === FnAnimation ===
// ===================

pub trait FnAnimation = FnMut(f32) + 'static;

// ====================
// === AnimatorData ===
// ====================

struct AnimatorData {
    closure          : Box<dyn FnMut(f32)>,
    previous_time    : Option<f32>,
    step_duration    : f32,
    accumulated_time : f32
}

impl AnimatorData {
    pub fn new<F:FnAnimation>(steps_per_second:f32, f:F) -> Self {
        let closure          = Box::new(f);
        let previous_time    = None;
        let step_duration    = 1.0 / steps_per_second;
        let accumulated_time = zero();
        Self { closure,previous_time,step_duration,accumulated_time }
    }
}

// ================
// === Animator ===
// ================

/// This structure is aimed to run a closure at a fixed time rate.
pub struct Animator {
    _animation_loop: AnimationFrameLoop
}

impl Animator {
    pub fn new<F:FnAnimation>(steps_per_second:f32, f:F) -> Self {
        let mut data        = AnimatorData::new(steps_per_second, f);
        let _animation_loop = AnimationFrameLoop::new(move |current_time| {
            if let Some(previous_time) = data.previous_time {
                let dt = (current_time - previous_time) / 1000.0;
                data.accumulated_time += dt;
                while data.accumulated_time > data.step_duration {
                    data.accumulated_time -= data.step_duration;
                    (data.closure)(data.step_duration);
                }
            }
            data.previous_time = Some(current_time);
        });
        Self { _animation_loop }
    }
}
