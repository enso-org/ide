//! This module implements physics components to simulate a rubber band dynamics.
//! The components has the potential to be further developed and extended in the future into a
//! more sophisticated physics simulator.

use crate::prelude::*;

use crate::animation::animator::Animator;
use crate::animation::animator::fixed_step::IntervalCounter;
use crate::animation::linear_interpolation;
use crate::control::event_loop::*;

use nalgebra::Vector3;
use nalgebra::*;



pub trait Magnitude {
    fn magnitude(&self) -> f32;
}

pub trait Normalize {
    fn normalize(&self) -> Self;
}



#[derive(Clone,Copy,Debug,Neg,Sub,Add,Div,AddAssign,From,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Position3 {
    pub vec : Vector3<f32>
}

impl Position3 {
    pub fn new(x:f32, y:f32, z:f32) -> Self {
        let vec = Vector3::new(x,y,z);
        Self {vec}
    }
}

impl Default for Position3 {
    fn default() -> Self {
        let vec = zero();
        Self {vec}
    }
}

impl Magnitude for Position3 {
    fn magnitude(&self) -> f32 {
        self.vec.magnitude()
    }
}

impl Normalize for Position3 {
    fn normalize(&self) -> Self {
        Self {vec:self.vec.normalize()}
    }
}

impl Mul<f32> for Position3 {
    type Output = Position3;
    fn mul(self, rhs:f32) -> Self::Output {
        let vec = self.vec * rhs;
        Self {vec}
    }
}

impl Into<Vector3<f32>> for Position3 {
    fn into(self) -> Vector3<f32> {
        self.vec
    }
}



// ==================
// === Properties ===
// ==================

macro_rules! define_property {
    ($(#$meta:tt)* $name:ident = $default:expr) => {
        $(#$meta)*
        #[derive(Debug,Clone,Copy,Into,From)]
        pub struct $name {
            /// Internal value of the $name.
            pub value : f32,
        }

        impl $name {
            /// Constructor.
            pub fn new() -> Self {
                default()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                let value = $default;
                Self {value}
            }
        }
    };
}

define_property! { Drag   = 1500.0 }
define_property! { Spring = 20000.0 }
define_property! { Mass   = 30.0 }



// ==================
// === Thresholds ===
// ==================

/// Thresholds defining the values which define when simulation stops.
#[derive(Clone,Copy,Debug)]
#[allow(missing_docs)]
pub struct Thresholds {
    pub distance : f32,
    pub speed    : f32
}

impl Default for Thresholds {
    fn default() -> Self {
        Self::new(0.1,0.1)
    }
}

impl Thresholds {
    /// Constructor.
    pub fn new(distance:f32, speed:f32) -> Self {
        Self {distance,speed}
    }
}



// ======================
// === SimulationData ===
// ======================

/// A fixed step physics simulator used to simulate `PhysicsState`.
#[derive(Clone,Copy,Debug,Default)]
pub struct SimulationData {
    position        : Position3,
    target_position : Position3,
    velocity        : Position3,
    mass            : Mass,
    spring          : Spring,
    drag            : Drag,
    thresholds      : Thresholds,
    active          : bool,
}

impl SimulationData {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Runs a simulation step.
    fn step(&mut self, delta_seconds:f32) {
        if self.active {
            let velocity      = self.velocity.magnitude();
            let distance      = (self.position - self.target_position).magnitude();
            let snap_velocity = velocity < self.thresholds.speed;
            let snap_distance = distance < self.thresholds.distance;
            let should_snap   = snap_velocity && snap_distance;
            if should_snap {
                self.position = self.target_position;
                self.velocity = default();
                self.active   = false;
            } else {
                let force        = self.spring_force() + self.drag_force();
                let acceleration = force / self.mass.value;
                self.velocity   += acceleration  * delta_seconds;
                self.position   += self.velocity * delta_seconds;
            }
        }
    }

    /// Compute spring force.
    fn spring_force(&self) -> Position3 {
        let position_delta = self.target_position - self.position;
        let distance       = position_delta.magnitude();
        if distance > 0.0 {
            let coefficient = distance * self.spring.value;
            position_delta.normalize() * coefficient
        } else {
            default()
        }
    }

    /// Compute air drag force.
    fn drag_force(&self) -> Position3 {
        -self.velocity * self.drag.value
    }
}


// === Getters ===

#[allow(missing_docs)]
impl SimulationData {
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn position(&self) -> Position3 {
        self.position
    }

    pub fn target_position(&self) -> Position3 {
        self.target_position
    }
}


// === Setters ===

#[allow(missing_docs)]
impl SimulationData {
    pub fn set_mass(&mut self, mass:Mass) {
        self.mass = mass;
    }

    pub fn set_spring(&mut self, spring:Spring) {
        self.spring = spring;
    }

    pub fn set_drag(&mut self, drag:Drag) {
        self.drag = drag;
    }

    pub fn set_position(&mut self, position:Position3) {
        self.active = true;
        self.position = position;
    }

    pub fn set_velocity(&mut self, velocity:Position3) {
        self.active = true;
        self.velocity = velocity;
    }

    pub fn set_target_position(&mut self, target_position:Position3) {
        self.active = true;
        self.target_position = target_position;
    }

    pub fn update_target_position<F:FnOnce(Position3)->Position3>(&mut self, f:F) {
        self.set_target_position(f(self.target_position()));
    }
}



// ==================
// === Simulation ===
// ==================

#[derive(Clone,Debug,Default)]
pub struct Simulation {
    data : Rc<Cell<SimulationData>>
}

impl CloneRef for Simulation {
    fn clone_ref(&self) -> Self {
        let data = self.data.clone_ref();
        Self {data}
    }
}

impl Simulation {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Runs a simulation step.
    pub fn step(&self, delta_seconds:f32) {
        let mut data = self.data.get();
        data.step(delta_seconds);
        self.data.set(data);
    }
}


// === Getters ===

#[allow(missing_docs)]
impl Simulation {
    pub fn active(&self) -> bool {
        self.data.get().active()
    }

    pub fn position(&self) -> Position3 {
        self.data.get().position()
    }

    pub fn target_position(&self) -> Position3 {
        self.data.get().target_position()
    }
}


// === Setters ===

#[allow(missing_docs)]
impl Simulation {
    pub fn set_mass(&self, mass:Mass) {
        self.data.update(|mut sim| {sim.set_mass(mass); sim});
    }

    pub fn set_spring(&self, spring:Spring) {
        self.data.update(|mut sim| {sim.set_spring(spring); sim});
    }

    pub fn set_drag(&self, drag:Drag) {
        self.data.update(|mut sim| {sim.set_drag(drag); sim});
    }

    pub fn set_velocity(&self, velocity:Position3) {
        self.data.update(|mut sim| {sim.set_velocity(velocity); sim});
    }

    pub fn set_position(&self, position:Position3) {
        self.data.update(|mut sim| {sim.set_position(position); sim});
    }

    pub fn set_target_position(&self, target_position:Position3) {
        self.data.update(|mut sim| {sim.set_target_position(target_position); sim});
    }

    pub fn update_target_position<F:FnOnce(Position3) -> Position3>(&self, f:F) {
        self.data.update(|mut sim| {sim.update_target_position(f); sim});
    }
}



// ========================
// === InertiaSimulator ===
// ========================

/// Handy alias for `InertiaSimulator` with a boxed closure callback.
pub type DynInertiaSimulator = InertiaSimulator<Box<dyn FnMut(Position3)>>;

#[derive(Derivative,Shrinkwrap)]
#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
pub struct InertiaSimulator<Callback> {
    #[shrinkwrap(main_field)]
    pub simulation     : Simulation,
    pub animation_loop : FixedFrameRateAnimationLoop<Step<Callback>>,
}

impl<Callback> CloneRef for InertiaSimulator<Callback> {
    fn clone_ref(&self) -> Self {
        let simulation     = self.simulation.clone_ref();
        let animation_loop = self.animation_loop.clone_ref();
        Self {simulation,animation_loop}
    }
}

impl<Callback> InertiaSimulator<Callback>
where Callback : FnMut(Position3)+'static {
    /// Constructor.
    pub fn new(callback:Callback) -> Self {
        let frame_rate     = 60.0;
        let simulation     = Simulation::new();
        let step           = step(&simulation,callback);
        let animation_loop = AnimationLoop::new_with_fixed_frame_rate(frame_rate,step);
        Self {simulation,animation_loop}
    }
}

pub type Step<Callback> = impl FnMut(TimeInfo);
fn step<Callback>(simulation:&Simulation, mut callback:Callback) -> Step<Callback>
where Callback : FnMut(Position3)+'static {
    let simulation = simulation.clone_ref();
    move |time:TimeInfo| {
        let delta_seconds = (time.frame_time / 1000.0) as f32;
        if simulation.active() {
            simulation.step(delta_seconds);
            callback(simulation.position());
        }
    }
}
