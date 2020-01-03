use crate::system::web::animation_frame_loop::AnimationFrameLoop;
use super::AnimationCallback;

use std::rc::Rc;
use std::cell::RefCell;



// ==================================
// === ContinuousTimeAnimatorData ===
// ==================================

struct ContinuousAnimatorData {
    callback          : Box<dyn AnimationCallback>,
    relative_start_ms : f32,
    absolute_start_ms : Option<f32>
}

impl ContinuousAnimatorData {
    fn new<F:AnimationCallback>(f:F) -> Self {
        let callback          = Box::new(f);
        let relative_start_ms = 0.0;
        let absolute_start_ms = None;
        Self {callback,relative_start_ms,absolute_start_ms}
    }

    fn set_time(&mut self, time:f32) {
        self.relative_start_ms = time;
        self.absolute_start_ms = None;
    }
}



// ==========================
// === ContinuousAnimator ===
// ==========================

/// This structure runs an animation with continuous time as its input.
pub struct ContinuousAnimator {
    _animation_loop : AnimationFrameLoop,
    data            : Rc<RefCell<ContinuousAnimatorData>>
}

impl ContinuousAnimator {
    pub fn new<F:AnimationCallback>(f:F) -> Self {
        let data            = Rc::new(RefCell::new(ContinuousAnimatorData::new(f)));
        let data_clone      = data.clone();
        let _animation_loop = AnimationFrameLoop::new(move |current_time| {
            let mut data : &mut ContinuousAnimatorData = &mut data_clone.borrow_mut();
            let absolute_start_ms = if let Some(absolute_start_ms) = data.absolute_start_ms {
                absolute_start_ms
            } else {
                data.absolute_start_ms = Some(current_time);
                current_time
            };
            (data.callback)(current_time - absolute_start_ms + data.relative_start_ms);
        });
        Self { _animation_loop, data }
    }
}


// === Setters ===

impl ContinuousAnimator {
    /// Sets the current playback time.
    pub fn set_time(&mut self, time:f32) {
        self.data.borrow_mut().set_time(time);
    }
}