use crate::prelude::*;

use crate::animation::Animator;

use nalgebra::Vector3;
use nalgebra::zero;
use crate::traits::HasPosition;

// ======================
// === DragProperties ===
// ======================

/// This structure contains air dragging properties.
#[derive(Default, Clone)]
pub struct DragProperties {
    amount : Rc<RefCell<f32>>
}

impl DragProperties {
    pub fn new(amount:f32) -> Self {
        let amount = Rc::new(RefCell::new(amount));
        Self { amount }
    }

    /// Gets dragging amount.
    pub fn amount(&self) -> f32 { *self.amount.borrow() }

    /// Sets dragging amount.
    pub fn set_amount(&mut self, amount:f32) {
        *self.amount.borrow_mut() = amount
    }
}

// ========================
// === SpringProperties ===
// ========================

struct SpringData {
    mass        : f32,
    coefficient : f32,
    fixed_point : Vector3<f32>
}

/// This structure contains spring physics properties.
#[derive(Clone)]
pub struct SpringProperties {
    data : Rc<RefCell<SpringData>>
}

impl Default for SpringProperties {
    fn default() -> Self {
        Self::new(zero(), zero(), zero())
    }
}

impl SpringProperties {
    pub fn new(mass:f32, coefficient:f32, fixed_point:Vector3<f32>) -> Self {
        let data = Rc::new(RefCell::new(SpringData { mass, coefficient, fixed_point }));
        Self { data }
    }

    /// Gets mass.
    pub fn mass(&self) -> f32 {
        self.data.borrow().mass
    }

    /// Sets mass.
    pub fn set_mass(&mut self, mass:f32) {
        self.data.borrow_mut().mass = mass
    }

    /// Gets coefficient.
    pub fn coefficient(&self) -> f32 {
        self.data.borrow().coefficient
    }

    /// Sets coefficient.
    pub fn set_coefficient(&mut self, coefficient:f32) {
        self.data.borrow_mut().coefficient = coefficient
    }

    /// Gets spring's fixed point.
    pub fn fixed_point(&self) -> Vector3<f32> {
        self.data.borrow().fixed_point
    }

    /// Sets spring's fixed point.
    pub fn set_fixed_point(&mut self, fixed_point:Vector3<f32>) {
        self.data.borrow_mut().fixed_point = fixed_point
    }
}

// ============================
// === KinematicProperties ===
// ============================

struct KinematicData {
    position     : Vector3<f32>,
    velocity     : Vector3<f32>,
    acceleration : Vector3<f32>
}

/// This structure contains kinematics properties.
#[derive(Clone)]
pub struct KinematicProperties {
    data : Rc<RefCell<KinematicData>>
}

impl Default for KinematicProperties {
    fn default() -> Self { Self::new(zero(), zero(), zero()) }
}

impl KinematicProperties {
    pub fn new(position:Vector3<f32>, velocity:Vector3<f32>, acceleration:Vector3<f32>) -> Self {
        let data = Rc::new(RefCell::new(KinematicData { position, velocity, acceleration }));
        Self { data }
    }

    /// Gets velocity.
    pub fn velocity(&self) -> Vector3<f32> {
        self.data.borrow().velocity
    }

    /// Sets velocity.
    pub fn set_velocity(&mut self, velocity:Vector3<f32>) {
        self.data.borrow_mut().velocity = velocity
    }

    /// Gets acceleration.
    pub fn acceleration(&self) -> Vector3<f32> {
        self.data.borrow().acceleration
    }

    /// Sets acceleration.
    pub fn set_acceleration(&mut self, acceleration:Vector3<f32>) {
        self.data.borrow_mut().acceleration = acceleration
    }
}

impl HasPosition for KinematicProperties {
    fn position(&self) -> Vector3<f32> {
        self.data.borrow().position
    }

    fn set_position(&mut self, position:Vector3<f32>) {
        self.data.borrow_mut().position = position
    }
}

// ======================
// === PropertiesData ===
// ======================

struct PropertiesData {
    kinematics : KinematicProperties,
    spring     : SpringProperties,
    drag       : DragProperties
}

impl PropertiesData {
    pub fn new
    (kinematics:KinematicProperties, spring:SpringProperties, drag:DragProperties) -> Self {
        Self { kinematics, spring, drag }
    }
}

// ==================
// === Properties ===
// ==================

/// A structure including kinematics, drag and spring properties.
#[derive(Clone)]
pub struct Properties {
    data : Rc<RefCell<PropertiesData>>
}

impl Properties {
    pub fn new
    (kinematics:KinematicProperties, spring:SpringProperties, drag:DragProperties) -> Self {
        let data = Rc::new(RefCell::new(PropertiesData::new(kinematics, spring, drag)));
        Self { data }
    }

    pub fn kinematics(&self) -> KinematicProperties { self.data.borrow().kinematics.clone() }

    pub fn set_kinematics(&mut self, kinematics:KinematicProperties) {
        self.data.borrow_mut().kinematics = kinematics
    }
    pub fn spring(&self) -> SpringProperties { self.data.borrow().spring.clone() }

    pub fn set_spring(&mut self, spring:SpringProperties) {
        self.data.borrow_mut().spring = spring
    }
    pub fn drag(&self) -> DragProperties { self.data.borrow().drag.clone() }

    pub fn set_drag(&mut self, drag:DragProperties) {
        self.data.borrow_mut().drag = drag
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
fn simulate_kinematics(properties:&mut KinematicProperties, dt:f32) {
    properties.set_velocity(properties.velocity() + properties.acceleration() * dt);
    properties.set_position(properties.position() + properties.velocity()     * dt);
}

/// Simulate dragging on `KinematicProperties`.
fn simulate_dragging(kinematics:&mut KinematicProperties, drag:&DragProperties) {
    let velocity = kinematics.velocity();
    let speed    = velocity.norm();
    kinematics.set_velocity(velocity / (1.0 + drag.amount() * speed));
}

/// Simulate spring on `KinematicProperties` attached to a fixed point.
fn simulate_spring(properties:&mut KinematicProperties, spring_properties:&SpringProperties) {
    let delta     = spring_properties.fixed_point() - properties.position();
    let delta_len = delta.magnitude();
    if delta_len > 0.0 {
        let force_val = delta_len * spring_properties.coefficient();
        let force     = delta.normalize() * force_val;
        properties.set_acceleration(force / spring_properties.mass());
    }
}

impl PhysicsSimulator {
    /// Simulates `Properties` on `object`.
    pub fn new<T>(mut object:T, properties:Properties) -> Self
    where T: HasPosition + 'static {
        let steps_per_second = 60.0;
        let _animator = Animator::new(steps_per_second, move |delta_time| {
            properties.kinematics().set_position(object.position());
            simulate_spring(&mut properties.kinematics(), &properties.spring());
            simulate_dragging(&mut properties.kinematics(), &properties.drag());
            simulate_kinematics(&mut properties.kinematics(), delta_time);
            object.set_position(properties.kinematics().position());
        });
        Self { _animator }
    }
}