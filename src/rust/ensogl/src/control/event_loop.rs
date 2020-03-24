//! This module contains implementation of `EventLoop`, a loop manager which runs a
//! `EventLoopCallback` once per frame.

use crate::prelude::*;

use crate::control::callback::CallbackMut;
use crate::control::callback::CallbackMutFn;
use crate::control::callback::CallbackMut1Fn;
use crate::control::callback::CallbackHandle;
use crate::control::callback::CallbackRegistry1;
use crate::system::web;
use crate::closure;

use wasm_bindgen::prelude::Closure;



// =========================
// === EventLoopCallback ===
// =========================

/// A callback to register in EventLoop, taking time_ms:f64 as its input.
pub trait EventLoopCallback = CallbackMut1Fn<f64>;



//// =================
//// === EventLoop ===
//// =================
//
///// Event loop system.
/////
///// It allows registering callbacks and firing them on demand. After a callback
///// is registered, a `CallbackHandle` is returned. The callback is automatically
///// removed as soon as the handle is dropped. You can also use the `forget`
///// method on the handle to make the callback registered forever, but beware
///// that it can easily lead to memory leaks.
//#[derive(Debug,Default,Clone)]
//pub struct EventLoop {
//    rc: Rc<RefCell<EventLoopData>>,
//}
//
//impl CloneRef for EventLoop {}
//
//impl EventLoop {
//    /// Create and start a new event loop.
//    pub fn new() -> Self {
//        Self::default().init()
//    }
//
//    /// Init the event loop.
//    fn init(self) -> Self {
//        let data = Rc::downgrade(&self.rc);
//        let main = move |time_ms| { data.upgrade().map(|t| t.borrow_mut().run(time_ms)); };
//        with(self.rc.borrow_mut(), |mut data| {
//            data.main = Some(Closure::new(main));
//            web::request_animation_frame(&data.main.as_ref().unwrap());
//        });
//        self
//    }
//
//    /// Add new callback. Returns `CallbackHandle` which when dropped, removes
//    /// the callback as well.
//    pub fn add_callback<F:EventLoopCallback>(&self, callback:F) -> CallbackHandle {
//        self.rc.borrow_mut().callbacks.add(Box::new(callback))
//    }
//
//    /// Sets a callback which is called when the loop started.
//    pub fn set_on_loop_started<F:CallbackMutFn>(&self, f:F) {
//        self.rc.borrow_mut().set_on_loop_started(f)
//    }
//
//    /// Sets a callback which is called when the loop finished.
//    pub fn set_on_loop_finished<F:CallbackMutFn>(&self, f:F) {
//        self.rc.borrow_mut().set_on_loop_finished(f);
//    }
//}
//
//
//
//// =====================
//// === EventLoopData ===
//// =====================
//
//trait RequestFrameCallback = FnMut(f64) + 'static;
//
///// The internal state of the `EventLoop`.
//#[derive(Derivative)]
//#[derivative(Debug)]
//pub struct EventLoopData {
//    main             : Option<Closure<dyn RequestFrameCallback>>,
//    callbacks        : CallbackRegistry1<f64>,
//    #[derivative(Debug="ignore")]
//    on_loop_started  : CallbackMut,
//    #[derivative(Debug="ignore")]
//    on_loop_finished : CallbackMut,
//    main_id          : i32,
//}
//
//impl Default for EventLoopData {
//    fn default() -> Self {
//        let main             = default();
//        let callbacks        = default();
//        let main_id          = default();
//        let on_loop_started  = Box::new(||{});
//        let on_loop_finished = Box::new(||{});
//        Self {main,callbacks,on_loop_started,on_loop_finished,main_id}
//    }
//}
//
//impl EventLoopData {
//    /// Create new instance.
//    pub fn run(&mut self, current_time_ms:f64) {
//        (self.on_loop_started)();
//        let callbacks   = &mut self.callbacks;
//        let callback_id = self.main.as_ref().map_or(default(), |main| {
//            callbacks.run_all(&current_time_ms);
//            web::request_animation_frame(main)
//        });
//        self.main_id = callback_id;
//        (self.on_loop_finished)();
//    }
//
//    /// Sets a callback which is called when the loop started.
//    pub fn set_on_loop_started<F:CallbackMutFn>(&mut self, f:F) {
//        self.on_loop_started = Box::new(f);
//    }
//
//    /// Sets a callback which is called when the loop finished.
//    pub fn set_on_loop_finished<F:CallbackMutFn>(&mut self, f:F) {
//        self.on_loop_finished = Box::new(f);
//    }
//}
//
//impl Drop for EventLoopData {
//    fn drop(&mut self) {
//        web::cancel_animation_frame(self.main_id);
//    }
//}

#[derive(Clone,Debug)]
pub struct EventLoop {
    frame_loop : AnimationLoop<Box<dyn FnMut(f64)>>,
    data       : Rc<RefCell<EventLoopData>>,
}

impl CloneRef for EventLoop {
    fn clone_ref(&self) -> Self {
        let frame_loop  = self.frame_loop.clone_ref();
        let data        = self.data.clone_ref();
        Self {frame_loop,data}
    }
}

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
    pub fn new() -> Self {
        let callbacks        = default();
        let on_loop_started  = Box::new(||{});
        let on_loop_finished = Box::new(||{});
        Self {callbacks,on_loop_started,on_loop_finished}
    }
}

impl EventLoop {
    pub fn new() -> Self {
        let data       = Rc::new(RefCell::new(EventLoopData::new()));
        let weak       = Rc::downgrade(&data);
        let frame_loop = AnimationLoop::new(Box::new(move |time| {
            weak.upgrade().for_each(|data| {
                let mut data_mut = data.borrow_mut();
                (&mut data_mut.on_loop_started)();
                data_mut.callbacks.run_all(&time);
                (&mut data_mut.on_loop_finished)();
            })
        }) as Box<dyn FnMut(f64)>);
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




// =====================
// === AnimationLoop ===
// =====================

// === Types ===

pub trait AnimationCallback = FnMut(f64) + 'static;


// === Definition ===

#[derive(Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
pub struct AnimationLoop<Callback> {
    data: Rc<RefCell<AnimationLoopData<Callback>>>,
}

impl<Callback> CloneRef for AnimationLoop<Callback> {}

impl<Callback> AnimationLoop<Callback>
where Callback : AnimationCallback {
    /// Create and start a new animation loop.
    pub fn new(callback:Callback) -> Self {
        let data      = Rc::new(RefCell::new(AnimationLoopData::new(callback)));
        let weak_data = Rc::downgrade(&data);
        let on_frame  = move |time| weak_data.upgrade().for_each(|t| t.borrow_mut().run(time));
        data.borrow_mut().on_frame = Some(Closure::new(on_frame));
        web::request_animation_frame(&data.borrow_mut().on_frame.as_ref().unwrap());
        Self {data}
    }
}

/// The internal state of the `AnimationLoop`.
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct AnimationLoopData<Callback> {
    #[derivative(Debug="ignore")]
    callback  : Callback,
    on_frame  : Option<Closure<dyn AnimationCallback>>,
    handle_id : i32,
}

impl<Callback> AnimationLoopData<Callback> {
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

impl<Callback> Drop for AnimationLoopData<Callback> {
    fn drop(&mut self) {
        web::cancel_animation_frame(self.handle_id);
    }
}



// ================
// === Animator ===
// ================

#[derive(Clone,Copy,Debug)]
pub struct TimeInfo {
    pub start      : f64,
    pub last_frame : f64,
    pub local      : f64,
}

impl TimeInfo {
    pub fn new() -> Self {
        default()
    }
}

impl Default for TimeInfo {
    fn default() -> Self {
        let start      = web::performance().now();
        let last_frame = 0.0;
        let local      = 0.0;
        Self {start,last_frame,local}
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
pub struct Animator<Callback> {
    animation_loop : AnimationLoop<OnFrame<Callback>>,
    time_info      : Rc<Cell<TimeInfo>>,
}

impl<Callback> Animator<Callback>
where Callback : FnMut(TimeInfo)+'static {
    pub fn new(callback:Callback) -> Self {
        let time_info      = Rc::new(Cell::new(TimeInfo::new()));
        let animation_loop = AnimationLoop::new(on_frame(callback,time_info.clone_ref()));
        Self {animation_loop,time_info}
    }
}

pub type OnFrame<Callback> = impl FnMut(f64);
fn on_frame<Callback>(mut callback:Callback, time_info_ref:Rc<Cell<TimeInfo>>) -> OnFrame<Callback>
where Callback : FnMut(TimeInfo) {
    move |current_time:f64| {
        let time_info  = time_info_ref.get();
        let start      = time_info.start;
        let last_frame = current_time - start - time_info.local;
        let local      = current_time - start;
        let time_info  = TimeInfo {start,last_frame,local};
        time_info_ref.set(time_info);
        callback(time_info);
    }
}
