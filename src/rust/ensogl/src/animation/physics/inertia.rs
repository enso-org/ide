//! This module implements physics components to simulate a rubber band dynamics.
//! The components has the potential to be further developed and extended in the future into a
//! more sophisticated physics simulator.

use crate::prelude::*;

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



// =============
// === Value ===
// =============

pub trait Value
    = 'static + Copy + Default + Debug + Magnitude + Normalize
    + Neg<       Output=Self>
    + Sub<Self , Output=Self>
    + Add<Self , Output=Self>
    + Div<f32  , Output=Self>
    + Mul<f32  , Output=Self>;



// ==================
// === Properties ===
// ==================

macro_rules! define_property {
    ($name:ident = $default:expr) => {
        /// Simulation property.
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
pub struct SimulationData<T> {
    position        : T,
    target_position : T,
    velocity        : T,
    mass            : Mass,
    spring          : Spring,
    drag            : Drag,
    thresholds      : Thresholds,
    active          : bool,
}

impl<T:Value> SimulationData<T> {
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
                self.velocity    = self.velocity + acceleration  * delta_seconds;
                self.position    = self.position + self.velocity * delta_seconds;
            }
        }
    }

    /// Compute spring force.
    fn spring_force(&self) -> T {
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
    fn drag_force(&self) -> T {
        -self.velocity * self.drag.value
    }
}


// === Getters ===

#[allow(missing_docs)]
impl<T:Value> SimulationData<T> {
    pub fn position        (&self) -> T          { self.position }
    pub fn target_position (&self) -> T          { self.target_position }
    pub fn velocity        (&self) -> T          { self.velocity }
    pub fn mass            (&self) -> Mass       { self.mass }
    pub fn spring          (&self) -> Spring     { self.spring }
    pub fn drag            (&self) -> Drag       { self.drag }
    pub fn thresholds      (&self) -> Thresholds { self.thresholds }
    pub fn active          (&self) -> bool       { self.active }
}


// === Setters ===

#[allow(missing_docs)]
impl<T:Value> SimulationData<T> {
    pub fn set_position(&mut self, position:T) {
        self.active = true;
        self.position = position;
    }

    pub fn set_target_position(&mut self, target_position:T) {
        self.active = true;
        self.target_position = target_position;
    }

    pub fn update_position<F:FnOnce(T)->T>(&mut self, f:F) {
        self.set_position(f(self.position()));
    }

    pub fn update_target_position<F:FnOnce(T)->T>(&mut self, f:F) {
        self.set_target_position(f(self.target_position()));
    }

    pub fn set_velocity   (&mut self, velocity:T)            { self.velocity = velocity; }
    pub fn set_mass       (&mut self, mass:Mass)             { self.mass = mass; }
    pub fn set_spring     (&mut self, spring:Spring)         { self.spring = spring; }
    pub fn set_drag       (&mut self, drag:Drag)             { self.drag = drag; }
    pub fn set_thresholds (&mut self, thresholds:Thresholds) { self.thresholds = thresholds; }

    pub fn update_velocity<F:FnOnce(T)->T>(&mut self, f:F) {
        self.set_velocity(f(self.velocity()));
    }

    pub fn update_mass<F:FnOnce(Mass)->Mass>(&mut self, f:F) {
        self.set_mass(f(self.mass()));
    }

    pub fn update_spring<F:FnOnce(Spring)->Spring>(&mut self, f:F) {
        self.set_spring(f(self.spring()));
    }

    pub fn update_drag<F:FnOnce(Drag)->Drag>(&mut self, f:F) {
        self.set_drag(f(self.drag()));
    }

    pub fn update_thresholds<F:FnOnce(Thresholds)->Thresholds>(&mut self, f:F) {
        self.set_thresholds(f(self.thresholds()));
    }
}



// ==================
// === Simulation ===
// ==================

#[derive(Clone,Debug,Default)]
pub struct Simulation<T:Copy> {
    data : Rc<Cell<SimulationData<T>>>
}

impl<T:Value> CloneRef for Simulation<T> {
    fn clone_ref(&self) -> Self {
        let data = self.data.clone_ref();
        Self {data}
    }
}

impl<T:Value> Simulation<T> {
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
impl<T:Value> Simulation<T> {
    pub fn active(&self) -> bool {
        self.data.get().active()
    }

    pub fn position(&self) -> T {
        self.data.get().position()
    }

    pub fn target_position(&self) -> T {
        self.data.get().target_position()
    }
}


// === Setters ===

#[allow(missing_docs)]
impl<T:Value> Simulation<T> {
    pub fn set_mass(&self, mass:Mass) {
        self.data.update(|mut sim| {sim.set_mass(mass); sim});
    }

    pub fn set_spring(&self, spring:Spring) {
        self.data.update(|mut sim| {sim.set_spring(spring); sim});
    }

    pub fn set_drag(&self, drag:Drag) {
        self.data.update(|mut sim| {sim.set_drag(drag); sim});
    }

    pub fn set_velocity(&self, velocity:T) {
        self.data.update(|mut sim| {sim.set_velocity(velocity); sim});
    }

    pub fn set_position(&self, position:T) {
        self.data.update(|mut sim| {sim.set_position(position); sim});
    }

    pub fn set_target_position(&self, target_position:T) {
        self.data.update(|mut sim| {sim.set_target_position(target_position); sim});
    }

    pub fn update_target_position<F:FnOnce(T)->T>(&self, f:F) {
        self.data.update(|mut sim| {sim.update_target_position(f); sim});
    }
}



// ========================
// === InertiaSimulator ===
// ========================

/// Handy alias for `InertiaSimulator` with a boxed closure callback.
pub type DynInertiaSimulator<T> = InertiaSimulator<T,Box<dyn Fn(Position3)>>;

#[derive(Derivative,Shrinkwrap)]
#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
pub struct InertiaSimulator<T:Value,Callback> {
    #[shrinkwrap(main_field)]
    simulation     : Simulation<T>,
    animation_loop : Rc<CloneCell<Option<FixedFrameRateAnimationLoop<Step<T,Callback>>>>>,
    frame_rate     : Rc<Cell<f64>>,
    #[derivative(Debug="ignore")]
    callback : Rc<Callback>,
}

impl<T:Value,Callback> CloneRef for InertiaSimulator<T,Callback> {
    fn clone_ref(&self) -> Self {
        let simulation     = self.simulation.clone_ref();
        let animation_loop = self.animation_loop.clone_ref();
        let frame_rate     = self.frame_rate.clone_ref();
        let callback       = self.callback.clone_ref();
        Self {simulation,animation_loop,frame_rate,callback}
    }
}

impl<T:Value,Callback> InertiaSimulator<T,Callback>
where Callback : Fn(T)+'static {
    /// Constructor.
    pub fn new(callback:Callback) -> Self {
        let frame_rate     = Rc::new(Cell::new(60.0));
        let callback       = Rc::new(callback);
        let simulation     = Simulation::new();
        let animation_loop = default();
        Self {simulation,animation_loop,frame_rate,callback} . init()
    }

    fn init(self) -> Self {
        self.start();
        self
    }

    fn start(&self) {
        if self.animation_loop.get().is_none() {
            let frame_rate     = self.frame_rate.get();
            let step           = step(&self);
            let animation_loop = AnimationLoop::new_with_fixed_frame_rate(frame_rate,step);
            self.animation_loop.set(Some(animation_loop));
        }
    }

    pub fn set_position(&self, position:T) {
        self.simulation.set_position(position);
        self.start();
    }

    pub fn set_target_position(&self, target_position:T) {
        self.simulation.set_target_position(target_position);
        self.start();
    }

    pub fn update_target_position<F:FnOnce(T)->T>(&self, f:F) {
        self.simulation.update_target_position(f);
        self.start();
    }
}

pub type Step<T,Callback> = impl Fn(TimeInfo);
fn step<T:Value,Callback>(simulator:&InertiaSimulator<T,Callback>) -> Step<T,Callback>
where Callback : Fn(T)+'static {
    let this = simulator.clone_ref();
    move |time:TimeInfo| {
        let delta_seconds = (time.frame_time / 1000.0) as f32;
        if this.simulation.active() {
            this.simulation.step(delta_seconds);
            (this.callback)(this.simulation.position());
        } else {
            this.animation_loop.set(None);
        }
    }
}
