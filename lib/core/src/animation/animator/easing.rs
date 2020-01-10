//! This file provides the implementation of EasingAnimator.

use super::ContinuousAnimator;
use crate::animation::position::HasPosition;
use crate::animation::easing::FnEasing;
use crate::control::event_loop::EventLoop;
use crate::math::utils::linear_interpolation;

use nalgebra::Vector3;
use nalgebra::clamp;
use std::rc::Rc;
use std::cell::RefCell;


// ==========================
// === EasingAnimatorData ===
// ==========================

struct EasingAnimatorData {
    object              : Box<dyn HasPosition>,
    origin_position     : Vector3<f32>,
    target_position     : Vector3<f32>,
    duration_seconds    : f64,
    continuous_animator : Option<ContinuousAnimator>
}



// ======================
// === EasingAnimator ===
// ======================

/// This struct animates from `origin_position` to `target_position` using easing functions.
pub struct EasingAnimator {
    data : Rc<RefCell<EasingAnimatorData>>
}

impl EasingAnimator {
    /// Creates an EasingAnimator using a `easing_function` to move `object` from
    /// `origin_position` to `target_position` in `duration_seconds`.
    pub fn new<T,F>
    (mut event_loop    : &mut EventLoop
    , easing_function  : F
    , object           : T
    , origin_position  : Vector3<f32>
    , target_position  : Vector3<f32>
    , duration_seconds : f64) -> Self
    where T : HasPosition + 'static, F : FnEasing {
        let continuous_animator = None;
        let object               = Box::new(object);
        let data                 = EasingAnimatorData{
            origin_position,
            target_position,
            object,
            duration_seconds,
            continuous_animator
        };
        let data = Rc::new(RefCell::new(data));
        let weak = Rc::downgrade(&data);
        let continuous_animator = ContinuousAnimator::new(&mut event_loop, move |time_ms| {
            if let Some(data) = weak.upgrade() {
                let mut data         = data.borrow_mut();
                let duration_seconds = data.duration_seconds;
                let origin_position  = data.origin_position;
                let target_position  = data.target_position;
                let time_seconds     = time_ms as f64 / 1000.0 / duration_seconds;
                let time_seconds     = clamp(time_seconds, 0.0, 1.0);
                let time_seconds     = easing_function(time_seconds) as f32;
                let position = linear_interpolation(origin_position, target_position, time_seconds);
                data.object.set_position(position);
            }
        });
        data.borrow_mut().continuous_animator = Some(continuous_animator);
        Self { data }
    }

    /// Animates the attached object from `origin_position` to `target_position` in
    /// `duration_seconds`.
    pub fn animate
    (&mut self, origin_position:Vector3<f32>, target_position:Vector3<f32>, duration_seconds:f64) {
        let mut data          = self.data.borrow_mut();
        data.origin_position  = origin_position;
        data.target_position  = target_position;
        data.duration_seconds = duration_seconds;
        data.continuous_animator.as_mut().map(|animator| animator.set_time(0.0));
    }
}
