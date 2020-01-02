// FIXME: Animators structs should get EventLoop as parameter. The whole application should have
// only one RequestAnimationFrame loop going on to avoid its overhead.

pub mod physics;
pub mod animator;



// ===================
// === FnAnimation ===
// ===================

pub trait FnAnimation = FnMut(f32) + 'static; // FIXME: This name does not tell what it is. When I see "FnAnimation" I understand that this is a "function that has something to do with animation". A much better name would be something like AnimatorCallback - it tells that it is a "callback used by Animator".


// FIXME: Please move things to better place then!
// FIXME: The objects in this section needs a better place.
// =============
// === Utils ===
// =============

use nalgebra::clamp;
use std::ops::Mul;
use std::ops::Add;

pub fn linear_interpolation<T>(a:T, b:T, t:f32) -> T
    where T : Mul<f32, Output = T> + Add<T, Output = T> {
    let t = clamp(t, 0.0, 1.0);
    a * (1.0 - t) + b * t
}
