//! This file provides several easing functions used for transition animation.

use core::f64::consts::PI;

/// Easing function signature.
pub trait FnEasing = Fn(f64) -> f64 + 'static;



// ========================
// === Easing functions ===
// ========================
// Reference: http://easings.net/en

macro_rules! easing_fn {
    (pub fn $name:ident(t:f64) -> f64 $block:block) => { paste::item! {
        /// A $name in transition.
        pub fn [<$name _in>](t:f64) -> f64 $block
        /// A $name out transition.
        pub fn [<$name _out>](t:f64) -> f64 { 1.0 - [<$name _in>](1.0 - t) }
        /// A $name in out transition.
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

/// A linear transition.
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

/// A back in transition with params.
pub fn back_in_params(t:f64, overshoot:f64) -> f64 { t * t * ((overshoot + 1.0) * t - overshoot) }

/// A back out transition with params.
pub fn back_out_params(t:f64, overshoot:f64) -> f64 {
    1.0 - back_in_params(1.0 - t, overshoot)
}

/// A back in out transition with params.
pub fn back_in_out_params(t:f64, overshoot:f64) -> f64 {
    let t = t * 2.0;
    if t < 1.0 {
        back_in_params(t, overshoot) / 2.0
    } else {
        (back_out_params(t - 1.0, overshoot) + 1.0) / 2.0
    }
}

easing_fn!(pub fn back(t:f64) -> f64 { back_in_params(t, 1.70158) });

/// A elastic in transition with params.
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

/// A elastic out transition with params.
pub fn elastic_out_params(t:f64, period:f64, amplitude:f64) -> f64 {
    1.0 - elastic_in_params(1.0 - t, period, amplitude)
}

/// A elastic in out transition with params.
pub fn elastic_in_out_params(t:f64, period:f64, amplitude:f64) -> f64 {
    let t = t * 2.0;
    if t < 1.0 {
        elastic_in_params(t, period, amplitude) / 2.0
    } else {
        (elastic_out_params(t - 1.0, period, amplitude) + 1.0) / 2.0
    }
}

easing_fn!(pub fn elastic(t:f64) -> f64 { elastic_in_params(t, 0.3, 1.0) });
