//! This module implements `ContinuousAnimator`, an object used to run a callback with a continuous
//! time in milliseconds as its input. It can be used to implement a playback mechanism.

use crate::prelude::*;

use crate::control::EventLoop;
use crate::control::callback::CallbackHandle;
use super::AnimationCallback;
use crate::system::web;

use std::rc::Rc;
use std::cell::RefCell;



// ================
// === TimeData ===
// ================

#[derive(Debug,Clone,Copy)]
pub struct TimeData {
    pub start    : f64,
    pub relative : f64,
}

impl TimeData {
    pub fn new() -> Self {
        let relative = 0.0;
        let start    = web::performance().now();
        Self {relative,start}
    }

    pub fn reset(&mut self) {
        self.set_relative_time(0.0);
    }

    pub fn set_relative_time(&mut self, time:f64) {
        self.start = web::performance().now() - time;
    }
}



// ==========================
// === ContinuousAnimator ===
// ==========================

/// `ContinuousAnimator` calls `AnimationCallback` with the playback time in millisecond as its
/// input once per frame.
#[derive(Clone,Derivative)]
#[derivative(Debug)]
pub struct ContinuousAnimator {
    event_loop : EventLoop,
    time       : Rc<Cell<TimeData>>,
    handle     : Rc<RefCell<Option<CallbackHandle>>>,
    #[derivative(Debug="ignore")]
    callback   : Rc<RefCell<Box<dyn AnimationCallback>>>,
}

impl ContinuousAnimator {
    /// Constructor.
    pub fn new<F:AnimationCallback>(f:F) -> Self {
        let callback      = Rc::new(RefCell::new(Box::new(f) as Box<dyn AnimationCallback>));
        let event_loop    = EventLoop::new();
        let time          = Rc::new(Cell::new(TimeData::new()));
        let weak_time     = Rc::downgrade(&time);
        let callback_weak = Rc::downgrade(&callback);
        let handle = event_loop.add_callback(move |current_time_ms:&f64| {
            weak_time.upgrade().map(|time| {
                callback_weak.upgrade().for_each(|f| {
                    let relative = current_time_ms - time.get().start;
                    let f_mut : &mut Box<dyn AnimationCallback> = &mut f.borrow_mut();
                    f_mut(relative);
                });
            });
        });
        let handle = Rc::new(RefCell::new(Some(handle)));
        Self {event_loop,time,callback,handle}
    }
}


// === Setters ===

impl ContinuousAnimator {
    /// Sets the current animator time.
    pub fn set_time(&mut self, time_ms:f64) {
        let mut time = self.time.get();
        time.set_relative_time(time_ms);
        self.time.set(time);
    }
}
