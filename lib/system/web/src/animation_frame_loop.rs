use super::request_animation_frame;

use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::Closure;


// =============================
// === AnimationLoopCallback ===
// =============================

pub trait AnimationLoopCallback = FnMut(f32) + 'static;



// ==========================
// === AnimationFrameData ===
// ==========================

// FIXME: We've got `control/event_loop.rs`. They do almost the same thing.
// They should be merged now or in the next code cleaning. In the later case,
// there should be a note about it.
struct AnimationFrameData {
    is_running : bool
}

pub struct AnimationFrameLoop {
    data : Rc<RefCell<AnimationFrameData>>
}



// ==========================
// === AnimationFrameLoop ===
// ==========================

impl AnimationFrameLoop {
    pub fn new<F:AnimationLoopCallback>(mut f:F) -> Self {
        let nop_func       = Box::new(|_| ());
        let nop_closure    = Closure::once(nop_func);
        let callback       = Rc::new(RefCell::new(nop_closure));
        let is_running     = true;
        let data           = AnimationFrameData { is_running };
        let data           = Rc::new(RefCell::new(data));
        let callback_clone = callback.clone();
        let data_clone     = data.clone();

        *callback.borrow_mut() = Closure::wrap(Box::new(move |delta_time| {
            if data_clone.borrow().is_running {
                f(delta_time);
                let callback = &callback_clone.borrow();
                request_animation_frame(&callback).expect("Request Animation \
                Frame");
            }
        }));
        request_animation_frame(&callback.borrow()).unwrap();
        AnimationFrameLoop{data}
    }
}

impl Drop for AnimationFrameLoop {
    fn drop(&mut self) {
        self.data.borrow_mut().is_running = false;
    }
}
