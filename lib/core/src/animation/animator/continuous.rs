use crate::system::web::get_performance;
use crate::system::web::animation_frame_loop::AnimationFrameLoop;
use super::FnAnimation;

use std::rc::Rc;
use std::cell::RefCell;


// ==================================
// === ContinuousTimeAnimatorData ===
// ==================================

struct ContinuousAnimatorData {
    closure      : Box<dyn FnMut(f32)>, // FIXME: this is a syntactic name, not a semantic one. Syntactic names answer the question "what it is: it is a closure". But they do not tell "what it is for". Is it a callback?
    start_time   : f32,                 // FIXME: Line above, you are using `dyn FnMut(f32)`, but below for the same parameter you use `FnAnimation`. This is inconsistency. Lets use the same type in every related place! :)
    current_time : f32 // FIXME: Why do you store this type? What it is used for ? I believe we can safely just delete it!
}

impl ContinuousAnimatorData {
    pub fn new<F:FnAnimation>(f:F) -> Self {
        let closure      = Box::new(f);
        let start_time   = get_performance().expect("Couldn't get performance timer").now() as f32; // FIXME: this is not a good place to call "expect". Calling expect in many places is bad idea, as it is a very big code duplication. Let's refactor it out :)
        let current_time = start_time;
        Self { closure,start_time,current_time } // FIXME: Spacing according to rules, should be: Self {closure,start_time,current_time}
    }

    // FIXME: Public functions should have documentation - please test your code with warn(missing_docs) - it will be mandatory soon, so its better to start adding docs now because later less code will need to be fixed :)
    pub fn set_time(&mut self, time:f32) {
        self.start_time  = self.current_time + time;
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
    pub fn new<F:FnAnimation>(f:F) -> Self {
        let data            = Rc::new(RefCell::new(ContinuousAnimatorData::new(f)));
        let data_clone      = data.clone();
        let _animation_loop = AnimationFrameLoop::new(move |current_time| {
            let mut data : &mut ContinuousAnimatorData = &mut data_clone.borrow_mut(); // FIXME: You don't need this type annotation here! :)
            (data.closure)(current_time - data.start_time);
            data.current_time = current_time;
        });
        Self { _animation_loop, data }
    }
}


// === Setters ===

// FIXME: this can be deleted.
impl ContinuousAnimator {
    pub fn set_time(&mut self, time:f32) {
        self.data.borrow_mut().set_time(time);
    }
}