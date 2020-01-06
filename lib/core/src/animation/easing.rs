use core::f64::consts::PI;
use basegl_system_web::animation_frame_loop::AnimationFrameLoop;
use crate::animation::HasPosition;
use nalgebra::Vector3;
use crate::animation::animator::continuous::ContinuousAnimator;
use crate::math::utils::linear_interpolation;
use nalgebra::clamp;
use crate::system::web::console_log;

pub trait FnEasing = Fn(f64) -> f64 + 'static;


// ==========================
// === EasingAnimatorData ===
// ==========================

// ======================
// === EasingAnimator ===
// ======================

/// This struct animates from `origin_position` to `target_position` using easing functions.
pub struct EasingAnimator {
    _continuous_animator : ContinuousAnimator
}

impl EasingAnimator {
    pub fn new<T,F>
    (mut event_loop   : &mut AnimationFrameLoop
     , easing_function  : F
     , mut object       : T
     , origin_position  : Vector3<f32>
     , target_position  : Vector3<f32>
     , duration_seconds : f64
    ) -> Self
    where T : HasPosition + 'static, F : FnEasing {
        let _continuous_animator = ContinuousAnimator::new(&mut event_loop, move |time_ms| {
            let time_seconds = time_ms as f64 / 1000.0 / duration_seconds;
            let time_seconds = clamp(time_seconds, 0.0, 1.0);
            let time_seconds = easing_function(time_seconds) as f32;
            let position = linear_interpolation(origin_position, target_position, time_seconds);
            object.set_position(position);
        });
        Self { _continuous_animator }
    }
}


// ========================
// === Easing functions ===
// ========================
// Reference: http://easings.net/en

macro_rules! easing_fn {
    (pub fn $name:ident(t:f64) -> f64 $block:block) => { paste::item! {
        pub fn [<$name _in>](t:f64) -> f64 $block
        pub fn [<$name _out>](t:f64) -> f64 { 1.0 - [<$name _in>](1.0 - t) }
        pub fn [<$name _in_out>](t:f64) -> f64 {
            let t = t * 2.0;
            if t < 1.0 {
                [<$name _in>](t) / 2.0
            } else {
                ([<$name _out>](t - 1.0) + 1.0) / 2.0
            }
}

    } };
}

easing_fn!(pub fn bounce(t:f64) -> f64 {
    if t < 1.0 / 2.75 { (7.5625 * t * t) }
    else if t < 2.0 / 2.75 {
        let t = t - 1.5 / 2.75;
        (7.5625 * t * t + 0.75)
    } else if t < 2.5 / 2.75 {
        let t = t - 2.25 / 2.75;
        (7.5625 * t * t + 0.9375)
    } else {
        let t = t - 2.625 / 2.75;
        (7.5625 * t * t + 0.984375)
    }
});

easing_fn!(pub fn circ(t:f64) -> f64 { 1.0 - (1.0 - t * t).sqrt() });

pub fn linear(t:f64) -> f64 { t }

easing_fn!(pub fn quad(t:f64) -> f64 { t * t });

easing_fn!(pub fn cubic(t:f64) -> f64 { t * t * t });

easing_fn!(pub fn quart(t:f64) -> f64 { t * t * t * t });

easing_fn!(pub fn quint(t:f64) -> f64 { t * t * t * t });

easing_fn!(pub fn expo(t:f64) -> f64 {
    if t == 0.0 {
        0.0
    } else {
        2.0_f64.powf(10.0 * (t - 1.0))
    }
});

easing_fn!(pub fn sine(t:f64) -> f64 { - (t * PI/2.0).cos() + 1.0 });

pub fn back_in_params(t:f64, overshoot:f64) -> f64 { t * t * ((overshoot + 1.0) * t - overshoot) }

pub fn back_out_params(t:f64, overshoot:f64) -> f64 {
    1.0 - back_in_params(1.0 - t, overshoot)
}

pub fn back_in_out_params(t:f64, overshoot:f64) -> f64 {
    let t = t * 2.0;
    if t < 1.0 {
        back_in_params(t, overshoot) / 2.0
    } else {
        (back_out_params(t - 1.0, overshoot) + 1.0) / 2.0
    }
}

easing_fn!(pub fn back(t:f64) -> f64 { back_in_params(t, 1.70158) });

pub fn elastic_in_params(t:f64, period:f64, amplitude:f64) -> f64 {
    let mut amplitude = amplitude;
    let overshoot     = if amplitude <= 1.0 {
        amplitude = 1.0;
        period / 4.0
    } else {
        period / (2.0 * PI) * (1.0 / amplitude).asin()
    };
    let elastic = amplitude * 2.0_f64.powf(-10.0 * t);
    elastic * ((t * 1.0 - overshoot) * (2.0 * PI) / period).sin() + 1.0
}

easing_fn!(pub fn elastic(t:f64) -> f64 { elastic_in_params(t, 0.3, 1.0) });
