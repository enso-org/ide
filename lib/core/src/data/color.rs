//! This module defines color structures and associated modifiers.

use crate::prelude::*;

pub use palette::rgb;
pub use palette::rgb::*;
pub use palette::encoding;

use crate::system::gpu::shader::glsl::Glsl;
use crate::system::gpu::shader::glsl::traits::*;



// ================
// === Gradient ===
// ================

/// A linear color gradient implementation. It accepts control points which base on colors for
/// a given palette. The palette need to define `mix` operation on GPU.
#[derive(Clone,Debug,Derivative)]
#[derivative(Default(bound=""))]
pub struct Gradient<Color> {
    control_points : Vec<(f32,Color)>,
}

impl<Color> Gradient<Color> {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Add a new control point. The offset needs to be in range [0..1].
    pub fn add(mut self, offset:f32, color:Color) -> Self {
        self.control_points.push((offset,color));
        self
    }
}

impls! { [Color] From<Gradient<Color>> for Glsl
where [Color:Copy+Into<Glsl>] {
    |t| {
        let args = t.control_points.iter().map(|(t,color)| {
            iformat!("gradient_control_point({t.glsl()},{(*color).glsl()})")
        }).join(",");
        iformat!("gradient({args})").into()
    }
}}



// ========================
// === DistanceGradient ===
// ========================

/// A gradient which transforms a linear gradient to a gradient along the signed distance field.
/// The slope parameter modifies how fast the gradient values are changed, allowing for nice,
/// smooth transitions.
#[derive(Copy,Clone,Debug)]
pub struct DistanceGradient<Gradient> {
    /// The distance from the shape border at which the gradient should start.
    pub min_distance : f32,
    /// The distance from the shape border at which the gradient should finish.
    pub max_distance : f32,
    /// The gradient slope modifier. Defines how fast the gradient values change.
    pub slope : Slope,
    /// The underlying gradient.
    pub gradient : Gradient
}

impl<Gradient> DistanceGradient<Gradient> {
    /// Constructor.
    pub fn new(gradient:Gradient) -> Self {
        let min_distance = 0.0;
        let max_distance = 10.0;
        let slope        = Slope::Smooth;
        Self {min_distance,max_distance,slope,gradient}
    }

    /// Setter for the `max_distance` field.
    pub fn max_distance(mut self, t:f32) -> Self {
        self.max_distance = t;
        self
    }
}


// === Instances ===

impl<Gradient> HasContent for DistanceGradient<Gradient> {
    type Content = Gradient;
}

impl<Gradient> Unwrap for DistanceGradient<Gradient> {
    fn unwrap(&self) -> &Self::Content {
        &self.gradient
    }
}

impls! {[G:Into<Glsl>] From<DistanceGradient<G>> for Glsl {
    |g| {
        let span   = iformat!("{g.max_distance.glsl()} - {g.min_distance.glsl()}");
        let offset = iformat!("shape.sdf.distance + {g.max_distance.glsl()}");
        let norm   = iformat!("({offset}) / ({span})");
        let t      = match g.slope {
            Slope::Linear => norm,
            Slope::Smooth => iformat!("smoothstep(0.0,1.0,{norm})"),
        };
        let expr   = iformat!("sample({g.gradient.glsl()},{t})");
        expr.into()
    }
}}

/// Defines how fast gradient values change.
#[derive(Copy,Clone,Debug)]
pub enum Slope {
    /// Defines a linear gradient.
    Linear,
    /// Perform Hermite interpolation between gradient values. See `GLSL` `smoothstep` for
    /// reference.
    Smooth,
}
