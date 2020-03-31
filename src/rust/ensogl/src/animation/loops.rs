//! This module contains implementation of `EventLoop`, a loop manager which runs a
//! `EventLoopCallback` once per frame.

use crate::prelude::*;

use crate::control::callback::CallbackMut;
use crate::control::callback::CallbackMutFn;
use crate::control::callback::CallbackMut1Fn;
use crate::control::callback::CallbackHandle;
use crate::control::callback::CallbackRegistry1;
use crate::system::web;

use wasm_bindgen::prelude::Closure;



// =========================
// === EventLoopCallback ===
// =========================

/// A callback to register in EventLoop, taking time_ms:f64 as its input.
pub trait EventLoopCallback = CallbackMut1Fn<f64>;

/// Event loop system.
///
/// It allows registering callbacks which will be fired on every animation frame. After a callback
/// is registered, a `CallbackHandle` is returned. The callback is automatically removed as soon as
/// its handle is dropped. You can also use the `forget` method on the handle to make the callback
/// registered forever, but beware that it can easily lead to memory leaks.
#[derive(Clone,CloneRef,Debug)]
pub struct EventLoop {
    frame_loop : RawLoop<Box<dyn FnMut(f64)>>,
    data       : Rc<RefCell<EventLoopData>>,
}

/// Internal representation for `EventLoop`.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct EventLoopData {
    callbacks        : CallbackRegistry1<f64>,
    #[derivative(Debug="ignore")]
    on_loop_started  : CallbackMut,
    #[derivative(Debug="ignore")]
    on_loop_finished : CallbackMut,
}

impl EventLoopData {
    /// Constructor.
    pub fn new() -> Self {
        let callbacks        = default();
        let on_loop_started  = Box::new(||{});
        let on_loop_finished = Box::new(||{});
        Self {callbacks,on_loop_started,on_loop_finished}
    }
}

impl EventLoop {
    /// Constructor.
    pub fn new() -> Self {
        let data = Rc::new(RefCell::new(EventLoopData::new()));
        let weak = Rc::downgrade(&data);
        let frame_loop :RawLoop<Box<dyn FnMut(f64)>> =
            RawLoop::new(Box::new(move |time| {
                weak.upgrade().for_each(|data| {
                    let mut data_mut = data.borrow_mut();
                    (&mut data_mut.on_loop_started)();
                    data_mut.callbacks.run_all(&time);
                    (&mut data_mut.on_loop_finished)();
                })
            }));
        Self {frame_loop,data}
    }

    /// Add new callback. Returns `CallbackHandle` which when dropped, removes
    /// the callback as well.
    pub fn add_callback<F:EventLoopCallback>(&self, callback:F) -> CallbackHandle {
        self.data.borrow_mut().callbacks.add(Box::new(callback))
    }

    /// Sets a callback which is called when the loop started.
    pub fn set_on_loop_started<F:CallbackMutFn>(&self, f:F) {
        self.data.borrow_mut().on_loop_started = Box::new(f);
    }

    /// Sets a callback which is called when the loop finished.
    pub fn set_on_loop_finished<F:CallbackMutFn>(&self, f:F) {
        self.data.borrow_mut().on_loop_finished = Box::new(f);
    }
}




// ===============
// === RawLoop ===
// ===============

// === Types ===

/// Callback for `RawLoop`.
pub trait RawLoopCallback = FnMut(f64) + 'static;


// === Definition ===

/// The most performant animation loop possible. However, if you are looking for a way to define
/// an animation loop, you are probably looking for the `Loop` which adds slight complexity
/// in order to provide better time information. The complexity is so small that it would not be
/// noticeable in almost any use case.
#[derive(Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
pub struct RawLoop<Callback> {
    data: Rc<RefCell<RawLoopData<Callback>>>,
}

impl<Callback> CloneRef for RawLoop<Callback> {}

impl<Callback> RawLoop<Callback>
where Callback : RawLoopCallback {
    /// Create and start a new animation loop.
    pub fn new(callback:Callback) -> Self {
        let data      = Rc::new(RefCell::new(RawLoopData::new(callback)));
        let weak_data = Rc::downgrade(&data);
        let on_frame  = move |time| weak_data.upgrade().for_each(|t| t.borrow_mut().run(time));
        data.borrow_mut().on_frame = Some(Closure::new(on_frame));
        let handle_id = web::request_animation_frame(&data.borrow_mut().on_frame.as_ref().unwrap());
        data.borrow_mut().handle_id = handle_id;
        Self {data}
    }
}

/// The internal state of the `RawLoop`.
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct RawLoopData<Callback> {
    #[derivative(Debug="ignore")]
    callback  : Callback,
    on_frame  : Option<Closure<dyn RawLoopCallback>>,
    handle_id : i32,
}

impl<Callback> RawLoopData<Callback> {
    /// Constructor.
    fn new(callback:Callback) -> Self {
        let on_frame  = default();
        let handle_id = default();
        Self {on_frame,callback,handle_id}
    }

    /// Run the animation frame.
    fn run(&mut self, current_time_ms:f64)
    where Callback:FnMut(f64) {
        let callback   = &mut self.callback;
        self.handle_id = self.on_frame.as_ref().map_or(default(), |on_frame| {
            callback(current_time_ms);
            web::request_animation_frame(on_frame)
        })
    }
}

impl<Callback> Drop for RawLoopData<Callback> {
    fn drop(&mut self) {
        web::cancel_animation_frame(self.handle_id);
    }
}



// ================
// === TimeInfo ===
// ================

/// Note: the `start` field will be computed on first run. We cannot compute it upfront, as other
/// time functions, like `performance.now()` can output nor precise results. The exact results
/// differ across browsers and browser versions. We have even observed that `performance.now()` can
/// sometimes provide a bigger value than time provided to `requestAnimationFrame` callback later,
/// which resulted in a negative frame time.
#[derive(Clone,Copy,Debug,Default)]
pub struct TimeInfo {
    /// Start time of the animation loop.
    pub start : f64,
    /// The last frame time.
    pub frame : f64,
    /// The time which passed since the animation loop was started.
    pub local : f64,
}

impl TimeInfo {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }
}



// ============
// === Loop ===
// ============

// === Types ===

pub trait LoopCallback = FnMut(TimeInfo) + 'static;


// === Definition ===

/// An animation loop. Runs the provided `Callback` every animation frame. It uses the
/// `RawLoop` under the hood. If you are looking for a more complex version where you can
/// register new callbacks for every frame, take a look at the ``.
#[derive(Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
pub struct Loop<Callback> {
    animation_loop : RawLoop<OnFrame<Callback>>,
    time_info      : Rc<Cell<TimeInfo>>,
}

impl<Callback> CloneRef for Loop<Callback> {
    fn clone_ref(&self) -> Self {
        let animation_loop = self.animation_loop.clone_ref();
        let time_info      = self.time_info.clone_ref();
        Self {animation_loop,time_info}
    }
}

impl<Callback> Loop<Callback>
where Callback : LoopCallback {
    pub fn new(callback:Callback) -> Self {
        let time_info      = Rc::new(Cell::new(TimeInfo::new()));
        let animation_loop = RawLoop::new(on_frame(callback,time_info.clone_ref()));
        Self {animation_loop,time_info}
    }
}

pub type OnFrame<Callback> = impl FnMut(f64);
fn on_frame<Callback>(mut callback:Callback, time_info_ref:Rc<Cell<TimeInfo>>) -> OnFrame<Callback>
where Callback : LoopCallback {
    move |current_time:f64| {
        let time_info = time_info_ref.get();
        let start     = if time_info.start == 0.0 {current_time} else {time_info.start};
        let frame     = current_time - start - time_info.local;
        let local     = current_time - start;
        let time_info = TimeInfo {start,frame,local};
        time_info_ref.set(time_info);
        callback(time_info);
    }
}



// =============================
// === FixedFrameRateSampler ===
// =============================

/// A callback `FnMut(TimeInfo) -> FnMut(TimeInfo)` transformer. Calls the inner callback with a
/// constant frame rate.
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct FixedFrameRateSampler<Callback> {
    frame_time  : f64,
    local_time  : f64,
    time_buffer : f64,
    #[derivative(Debug="ignore")]
    callback    : Callback,
}

impl<Callback> FixedFrameRateSampler<Callback> {
    pub fn new(frame_rate:f64, callback:Callback) -> Self {
        let frame_time  = 1000.0 / frame_rate;
        let local_time  = default();
        let time_buffer = default();
        Self {frame_time,local_time,time_buffer,callback}
    }
}

impl<Callback:FnOnce<(TimeInfo,)>> FnOnce<(TimeInfo,)> for FixedFrameRateSampler<Callback> {
    type Output = ();
    extern "rust-call" fn call_once(self, args:(TimeInfo,)) -> Self::Output {
        self.callback.call_once(args);
    }
}

impl<Callback:FnMut<(TimeInfo,)>> FnMut<(TimeInfo,)> for FixedFrameRateSampler<Callback> {
    extern "rust-call" fn call_mut(&mut self, args:(TimeInfo,)) -> Self::Output {
        let time = args.0;
        self.time_buffer += time.frame;
        loop {
            if self.time_buffer < 0.0 {
                break
            } else {
                self.time_buffer -= self.frame_time;
                let start = time.start;
                let frame = self.frame_time;
                let local = self.local_time;
                let time2 = TimeInfo {start,frame,local};
                self.local_time += self.frame_time;
                self.callback.call_mut((time2,));
            }
        }
    }
}



// ==========================
// === FixedFrameRateLoop ===
// ==========================

pub type FixedFrameRateLoop<Callback> = Loop<FixedFrameRateSampler<Callback>>;

impl<Callback> FixedFrameRateLoop<Callback>
where Callback:LoopCallback {
    pub fn new_with_fixed_frame_rate(frame_rate:f64, callback:Callback) -> Self {
        Self::new(FixedFrameRateSampler::new(frame_rate,callback))
    }
}
