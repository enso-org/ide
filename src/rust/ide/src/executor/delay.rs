//! Module defining `FrameDelay` - a `Future` that wraps a closure that should be called at a later
//! time.

pub use enso_prelude::*;

use enso_protocol::prelude::Future;
use ensogl_system_web::prelude::Closure;
use ensogl_system_web::request_animation_frame;
use futures::task::Context;
use futures::task::Poll;
use futures::task::Waker;
use std::pin::Pin;


// ========================
// === Type Definitions ===
// ========================

type OnFrameClosure = Closure<dyn FnMut(f64)>;



// ===================
// === Frame Delay ===
// ===================


/// Delayed calls to a closure. The time delays is specified in number of render frames.
#[derive(Debug)]
pub struct FrameDelay<T>{
    /// Counter that keeps track of the passage of frames. Needs to be shared with the JS runtime.
    frames_left : Rc<Cell<u64>>,
    /// Internal closure that is shared with the JS runtime and updates the `frames_left`.
    on_frame    : Option<OnFrameClosure>,
    /// Target callback that is called after the delay has passed.
    target      : T,
}

impl<T> Unpin for FrameDelay<T>{}

impl<T> FrameDelay<T> {
    /// Constructor. Takes the number of render frames and the closure that should be called after
    /// the render frames have been called.
    pub fn new(frames:u64, target:T) -> Self {
        let frames_left = Rc::new(Cell::new(frames));
        let on_frame    = None;
        FrameDelay{frames_left,target,on_frame}
    }

    /// Return a reference to the `on_frame` Closure. Creates it if it has not been created.
    fn on_frame_action_init_if_uninit(&mut self, waker:&Waker) -> &OnFrameClosure {
        if self.on_frame.is_none() {
            let waker       = waker.clone();
            let frames_left = Rc::clone(&self.frames_left);
            let closure     = Closure::new(move |_:f64| {
                let decreased = frames_left.get().saturating_sub(1);
                frames_left.set(decreased);
                waker.clone().wake();
            });
            self.on_frame = Some(closure);
        }
        // Panic Note: initialisation above cannot fail, so this will never panic.
        self.on_frame.as_ref().expect("`on_frame` was not initialised.")
    }

    /// Set the Js callback to be called on the next frame. Needs to be called on every frame.
    fn set_on_frame_action(&mut self, waker:&Waker) {
        let closure  = self.on_frame_action_init_if_uninit(waker);
        request_animation_frame(closure);
    }
}

impl<T:Fn()> Future for FrameDelay<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.frames_left.get() > 0 {
            self.get_mut().set_on_frame_action(cx.waker());
            return Poll::Pending
        }
        (self.target)();
        Poll::Ready(())
    }
}
