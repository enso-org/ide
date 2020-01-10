//! This file provides the implementation of EasingAnimator.

use super::ContinuousAnimator;
use crate::animation::easing::FnEasing;
use crate::control::event_loop::EventLoop;
use crate::math::utils::linear_interpolation;
use crate::math::utils::Interpolable;

use nalgebra::clamp;
use std::rc::Rc;
use std::cell::RefCell;



// ===============================
// === EasingAnimationCallback ===
// ===============================

pub trait EasingAnimationCallback<T> = FnMut(T) + 'static;



// ==========================
// === EasingAnimatorData ===
// ==========================

struct EasingAnimatorData<T:Interpolable<T>+'static> {
    initial_value       : T,
    final_value         : T,
    duration_seconds    : f64,
    continuous_animator : Option<ContinuousAnimator>
}



// ======================
// === EasingAnimator ===
// ======================

/// This struct animates from `origin_position` to `target_position` using easing functions.
pub struct EasingAnimator<T:Interpolable<T>+'static> {
    data : Rc<RefCell<EasingAnimatorData<T>>>
}

impl<T:Interpolable<T>+'static> EasingAnimator<T> {
    /// Creates an EasingAnimator using a `easing_function` to move `object` from
    /// `initial_value` to `final_value` in `duration_seconds`.
    pub fn new<F,C>
    ( mut event_loop                : &mut EventLoop
    , mut easing_animation_callback : C
    , easing_function               : F
    , initial_value                 : T
    , final_value                   : T
    , duration_seconds              : f64) -> Self
    where F:FnEasing, C:EasingAnimationCallback<T> {
        let continuous_animator = None;
        let data                 = EasingAnimatorData{
            initial_value,
            final_value,
            duration_seconds,
            continuous_animator
        };
        let data = Rc::new(RefCell::new(data));
        let weak = Rc::downgrade(&data);
        let continuous_animator = ContinuousAnimator::new(&mut event_loop, move |time_ms| {
            if let Some(data) = weak.upgrade() {
                let data             = data.borrow();
                let duration_seconds = data.duration_seconds;
                let initial_value    = data.initial_value;
                let final_value      = data.final_value;
                let time_seconds     = time_ms as f64 / 1000.0 / duration_seconds;
                let time_seconds     = clamp(time_seconds, 0.0, 1.0);
                let time_seconds     = easing_function(time_seconds) as f32;
                let value = linear_interpolation(initial_value, final_value, time_seconds);
                easing_animation_callback(value);
            }
        });
        data.borrow_mut().continuous_animator = Some(continuous_animator);
        Self { data }
    }

    /// Animates the attached object from `initial_value` to `final_value` in
    /// `duration_seconds`.
    pub fn animate
    (&mut self, initial_value:T, final_value:T, duration_seconds:f64) {
        let mut data          = self.data.borrow_mut();
        data.initial_value    = initial_value;
        data.final_value      = final_value;
        data.duration_seconds = duration_seconds;
        data.continuous_animator.as_mut().map(|animator| animator.set_time(0.0));
    }
}
