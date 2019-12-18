use crate::prelude::*;

use nalgebra::Vector3;
use nalgebra::zero;
use nalgebra::clamp;

// ============================
// === KinematicProperties ===
// ============================

/// Kinematic properties such as position, velocity and acceleration.
#[derive(Debug)]
pub struct KinematicProperties {
    pub position     : Vector3<f32>,
    pub velocity     : Vector3<f32>,
    pub acceleration : Vector3<f32>
}

impl Default for KinematicProperties {
    fn default() -> Self {
        let position     = zero();
        let velocity     = zero();
        let acceleration = zero();
        Self { position, velocity, acceleration }
    }
}

impl KinematicProperties {
    pub fn new(position:Vector3<f32>, velocity:Vector3<f32>, acceleration:Vector3<f32>) -> Self {
        Self { position, velocity, acceleration }
    }
}

// ========================
// === PhysicsSimulator ===
// ========================

/// A physics kinematics simulator used to simulate `KinematicProperties`.
pub struct PhysicsSimulator {}

impl Default for PhysicsSimulator {
    fn default() -> Self {
        Self {}
    }
}

impl PhysicsSimulator {
    pub fn new() -> Self { default() }

    /// Simulate the `KinematicProperties`.
    pub fn simulate_kinematics(&self, properties:&mut KinematicProperties, dt:f32) {
        properties.velocity += properties.acceleration * dt;
        properties.position += properties.velocity     * dt;
    }

    /// Simulate dragging on `KinematicProperties`.
    pub fn simulate_dragging(&self, properties:&mut KinematicProperties, drag:f32, dt:f32) {
        properties.velocity *= clamp(1.0 - dt * drag, 0.0, 1.0);
    }

    /// Simulate spring on `KinematicProperties` attached to a fixed point.
    pub fn simulate_spring
    (&self, properties:&mut KinematicProperties, fixed:Vector3<f32>, mass:f32, coefficient:f32) {
        let delta     = fixed - properties.position;
        let delta_len = delta.magnitude();
        if delta_len > 0.0 {
            let force_val           = delta_len * coefficient;
            let force               = delta.normalize() * force_val;
            properties.acceleration = force / mass;
        }
    }
}