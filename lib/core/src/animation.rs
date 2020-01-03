use nalgebra::Vector3;

// FIXME: Animators structs should get EventLoop as parameter. The whole application should have
// only one RequestAnimationFrame loop going on to avoid its overhead.

pub mod physics;
pub mod animator;



// =========================
// === AnimationCallback ===
// =========================

pub trait AnimationCallback = FnMut(f32) + 'static;



// ===================
// === HasPosition ===
// ===================

pub trait HasPosition {
    /// Gets self's position.
    fn position(&self) -> Vector3<f32>;
    /// Sets self's position.
    fn set_position(&mut self, position:Vector3<f32>);
}