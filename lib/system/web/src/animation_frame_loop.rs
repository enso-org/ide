use super::request_animation_frame;

use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::Closure;


// =======================
// === FnAnimationLoop ===
// =======================

pub trait FnAnimationLoop = FnMut(f32) + 'static; // FIXME: What is the difference between FnAnimationLoop and FnAnimation? The names are very enigmatic



// ==========================
// === AnimationFrameData ===
// ==========================

// FIXME: We've got `control/event_loop.rs`. They do almost the same thing. They should be merged now or in the next code cleaning. In the later case, there should be a note about it.
struct AnimationFrameData {
    run : bool // FIXME: What is this variable about? It sounds like a method, but stores boolean. Ok, after reading the code, I understood it. It should be named `is_running`.
}

pub struct AnimationFrameLoop {
    data   : Rc<RefCell<AnimationFrameData>> // FIXME: spacing
}



// ==========================
// === AnimationFrameLoop ===
// ==========================

impl AnimationFrameLoop {
    pub fn new<F:FnAnimationLoop>(mut f:F) -> Self {
        let nop_func       = Box::new(|_| ()) as Box<dyn FnMut(f32)>; // FIXME: What is this cast for?
        let nop_closure    = Closure::once(nop_func);
        let callback       = Rc::new(RefCell::new(nop_closure));
        let run            = true;
        let data           = Rc::new(RefCell::new(AnimationFrameData { run }));
        let callback_clone = callback.clone();
        let data_clone     = data.clone();

        *callback.borrow_mut() = Closure::wrap(Box::new(move |delta_time| {
            if data_clone.borrow().run {
                f(delta_time);
                let clb = &callback_clone.borrow(); // FIXME: This var name is so short without a reason.
                request_animation_frame(&clb).expect("Request Animation Frame");
            }
        }) as Box<dyn FnMut(f32)>); // FIXME: what is this cast for?
        request_animation_frame(&callback.borrow()).unwrap();

        AnimationFrameLoop{data} // FIXME: spacing
    }
}

impl Drop for AnimationFrameLoop {
    fn drop(&mut self) {
        self.data.borrow_mut().run = false;
    }
}
