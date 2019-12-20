use crate::prelude::*;

use crate::animation::Animator;

use nalgebra::Vector3;
use nalgebra::zero;
use crate::traits::HasPosition;

// ======================
// === DragProperties ===
// ======================

/// This structure contains air dragging properties.
#[derive(Default, Clone, Copy)]
pub struct DragProperties {
    amount : f32
}

impl DragProperties {
    pub fn new(amount:f32) -> Self {
        Self { amount }
    }
}

// === Getters ===

impl DragProperties {
    pub fn amount(&self) -> f32 {
        self.amount
    }
}

// === Setters ===

impl DragProperties {
    pub fn set_amount(&mut self, amount:f32) {
        self.amount = amount
    }
}

// ========================
// === SpringProperties ===
// ========================

/// This structure contains spring physics properties.
#[derive(Debug, Clone, Copy)]
pub struct SpringProperties {
    coefficient : f32,
    fixed_point : Vector3<f32>
}

impl Default for SpringProperties {
    fn default() -> Self {
        Self::new(zero(),zero())
    }
}

impl SpringProperties {
    pub fn new(coefficient:f32, fixed_point:Vector3<f32>) -> Self {
        Self { coefficient,fixed_point }
    }
}

// === Getters ===

impl SpringProperties {
    pub fn coefficient(&self) -> f32          { self.coefficient }
    pub fn fixed_point(&self) -> Vector3<f32> { self.fixed_point }
}

// === Setters ===

impl SpringProperties {
    pub fn set_coefficient(&mut self, coefficient:f32)          { self.coefficient = coefficient }
    pub fn set_fixed_point(&mut self, fixed_point:Vector3<f32>) { self.fixed_point = fixed_point }
}

// ============================
// === KinematicProperties ===
// ============================

/// This structure contains kinematics properties.
#[derive(Debug, Clone, Copy)]
pub struct KinematicsProperties {
    position     : Vector3<f32>,
    velocity     : Vector3<f32>,
    acceleration : Vector3<f32>
}

impl Default for KinematicsProperties {
    fn default() -> Self { Self::new(zero(), zero(), zero()) }
}

impl KinematicsProperties {
    pub fn new(position:Vector3<f32>, velocity:Vector3<f32>, acceleration:Vector3<f32>) -> Self {
        Self { position,velocity,acceleration }
    }
}

// === Getters ===

impl KinematicsProperties {
    pub fn velocity    (&self) -> Vector3<f32> { self.velocity }
    pub fn acceleration(&self) -> Vector3<f32> { self.acceleration }
}

// === Setters ===

impl KinematicsProperties {
    pub fn set_velocity(&mut self, velocity:Vector3<f32>) {
        self.velocity = velocity
    }

    pub fn set_acceleration(&mut self, acceleration:Vector3<f32>) {
        self.acceleration = acceleration
    }
}

impl HasPosition for KinematicsProperties {
    fn position    (&self) -> Vector3<f32>            { self.position }
    fn set_position(&mut self, position:Vector3<f32>) { self.position = position }
}

// =============================
// === PhysicsPropertiesData ===
// =============================

struct PhysicsPropertiesData {
    kinematics : KinematicsProperties,
    spring     : SpringProperties,
    drag       : DragProperties
}

impl PhysicsPropertiesData {
    pub fn new
    (kinematics: KinematicsProperties, spring:SpringProperties, drag:DragProperties) -> Self {
        Self { kinematics,spring,drag }
    }
}

// =========================
// === PhysicsProperties ===
// =========================

/// A structure including kinematics, drag and spring properties.
#[derive(Clone)]
pub struct PhysicsProperties {
    data : Rc<RefCell<PhysicsPropertiesData>>
}

impl PhysicsProperties {
    pub fn new
    (kinematics: KinematicsProperties, spring:SpringProperties, drag:DragProperties) -> Self {
        let data = Rc::new(RefCell::new(PhysicsPropertiesData::new(kinematics, spring, drag)));
        Self { data }
    }
}

// === Getters ===

impl PhysicsProperties {
    pub fn kinematics(&self) -> KinematicsProperties { self.data.borrow().kinematics.clone() }
    pub fn spring    (&self) -> SpringProperties     { self.data.borrow().spring.clone() }
    pub fn drag      (&self) -> DragProperties       { self.data.borrow().drag.clone() }
}

// === Setters ===

impl PhysicsProperties {
    pub fn mod_kinematics<F:FnOnce(&mut KinematicsProperties)>(&mut self, f:F) {
        f(&mut self.data.borrow_mut().kinematics)
    }

    pub fn mod_spring<F:FnOnce(&mut SpringProperties)>(&mut self, f:F) {
        f(&mut self.data.borrow_mut().spring)
    }

    pub fn mod_drag<F:FnOnce(&mut DragProperties)>(&mut self, f:F) {
        f(&mut self.data.borrow_mut().drag)
    }
}

// ========================
// === SimulationObject ===
// ========================

pub trait SimulationObject = HasPosition + 'static;

// =====================
// === PhysicsObject ===
// =====================

/// This represents a physics objects with mass and a generic object with `HasPosition`.
pub struct PhysicsObject {
    object : Box<dyn SimulationObject>,
    mass   : f32
}

impl PhysicsObject {
    pub fn new<T:SimulationObject>(object:T, mass:f32) -> Self {
        let object = Box::new(object);
        Self { object, mass }
    }
}

// ========================
// === PhysicsSimulator ===
// ========================

/// A 60 steps per second physics simulator used to simulate `Properties`.
pub struct PhysicsSimulator {
    _animator : Animator
}

/// Simulate the `KinematicProperties`.
fn simulate_kinematics(properties:&mut KinematicsProperties, dt:f32) {
    properties.set_velocity(properties.velocity() + properties.acceleration() * dt);
    properties.set_position(properties.position() + properties.velocity()     * dt);
}

/// Simulate dragging on `KinematicProperties`.
fn simulate_dragging(kinematics:&mut KinematicsProperties, drag:&DragProperties) {
    let velocity = kinematics.velocity();
    let speed    = velocity.norm();
    kinematics.set_velocity(velocity / (1.0 + drag.amount() * speed));
}

/// Simulate spring on `KinematicProperties` attached to a fixed point.
fn simulate_spring
(properties:&mut KinematicsProperties, spring_properties:&SpringProperties, mass:f32) {
    let delta     = spring_properties.fixed_point() - properties.position();
    let delta_len = delta.magnitude();
    if delta_len > 0.0 {
        let force_val = delta_len * spring_properties.coefficient();
        let force     = delta.normalize() * force_val;
        properties.set_acceleration(force / mass);
    }
}

impl PhysicsSimulator {
    /// Simulates `Properties` on `object`.
    pub fn new(mut object:PhysicsObject, mut properties:PhysicsProperties) -> Self {
        let steps_per_second = 60.0;
        let _animator = Animator::new(steps_per_second, move |delta_time| {
            let mass   = object.mass;
            let spring = properties.spring();
            let drag   = properties.drag();
            let object = &mut object.object;
            properties.mod_kinematics(|mut kinematics| {
                kinematics.set_position(object.position());
                simulate_spring(&mut kinematics, &spring, mass);
                simulate_dragging(&mut kinematics, &drag);
                simulate_kinematics(&mut kinematics, delta_time);
            });
            object.set_position(properties.kinematics().position());
        });
        Self { _animator }
    }
}
