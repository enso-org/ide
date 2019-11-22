use super::request_animation_frame;
use wasm_bindgen::prelude::Closure;
use std::rc::Rc;
use std::cell::RefCell;


// ============================
// === Animation Frame Data ===
// ============================

struct AnimationFrameData {
    run : bool
}

pub struct AnimationFrameLoop {
    forget : bool,
    data   : Rc<RefCell<AnimationFrameData>>
}

// ============================
// === Animation Frame Loop ===
// ============================

impl AnimationFrameLoop {
    pub fn new(mut func : Box<dyn FnMut()>) -> Self {
        let nop_func    = Box::new(|| ()) as Box<dyn FnMut()>;
        let nop_closure = Closure::wrap(nop_func);
        let callback    = Rc::new(RefCell::new(nop_closure));

        let run  = true;
        let data = Rc::new(RefCell::new(AnimationFrameData { run }));

        let callback_clone = callback.clone();
        let data_clone     = data.clone();

        *callback.borrow_mut() = Closure::wrap(Box::new(move || {
            if data_clone.borrow().run {
                func();
                request_animation_frame(&callback_clone.borrow())
                                       .expect("Request Animation Frame");
            }
        }) as Box<dyn FnMut()>);
        request_animation_frame(&callback.borrow()).unwrap();

        let forget = false;
        AnimationFrameLoop { forget, data }
    }

    pub fn forget(mut self) {
        self.forget = true;
    }
}

impl Drop for AnimationFrameLoop {
    fn drop(&mut self) {
        if !self.forget {
            self.data.borrow_mut().run = false
        }
    }
}
