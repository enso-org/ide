#![allow(missing_docs)]

/// This module defines color structures and associated modifiers.


use crate::prelude::*;

pub use palette::rgb;
pub use palette::rgb::*;
pub use palette::encoding;

use crate::system::gpu::shader::glsl::Glsl;
use crate::system::gpu::shader::glsl::traits::*;


//// ===============
//// === Sampler ===
//// ===============



#[derive(Clone,Debug,Derivative)]
#[derivative(Default(bound=""))]
pub struct Gradient<Color> {
    pub control_points : Vec<(f32,Color)>,
}

impl<Color> Gradient<Color> {
    pub fn new() -> Self {
        default()
    }

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

#[derive(Copy,Clone,Debug)]
pub struct DistanceGradient<Gradient> {
    pub min_distance : f32,
    pub max_distance : f32,
    pub slope        : Slope,
    pub gradient     : Gradient
}

impl<Gradient> DistanceGradient<Gradient> {
    pub fn new(gradient:Gradient) -> Self {
        let min_distance = 0.0;
        let max_distance = 10.0;
        let slope        = Slope::Smooth;
        Self {min_distance,max_distance,slope,gradient}
    }

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

#[derive(Copy,Clone,Debug)]
pub enum Slope {
    Linear,
    Smooth,
}
