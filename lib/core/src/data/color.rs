use crate::prelude::*;

//pub use palette::*;
pub use palette::rgb;
pub use palette::rgb::*;
pub use palette::encoding;

use crate::system::gpu::shader::glsl::Glsl;
use crate::system::gpu::shader::glsl::traits::*;


//// ===============
//// === Sampler ===
//// ===============
//
//trait Sampler {
//    type Output;
//    fn sample(&self, t:f32) -> Self::Output;
//}



#[derive(Clone,Debug)]
pub struct Gradient<Color> {
    pub control_points : Vec<(f32,Color)>,
}

impl<Color> Gradient<Color> {
    pub fn new(control_points:Vec<(f32,Color)>) -> Self {
        Self {control_points}
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
    pub gradient     : Gradient
}

impl<Gradient> DistanceGradient<Gradient> {
    pub fn new(min_distance:f32, max_distance:f32, gradient:Gradient) -> Self {
        Self {min_distance,max_distance,gradient}
    }
}

impl<Gradient> HasContent for DistanceGradient<Gradient> {
    type Content = Gradient;
}

impl<Gradient> Unwrap for DistanceGradient<Gradient> {
    fn unwrap(&self) -> &Self::Content {
        &self.gradient
    }
}


impls! {[G:Into<Glsl>] From<DistanceGradient<G>> for Glsl {
    |t| {
        let span   = iformat!("{t.max_distance.glsl()} - {t.min_distance.glsl()}");
        let offset = iformat!("shape.sdf.distance - {t.min_distance.glsl()}");
        let norm   = iformat!("({offset}) / ({span})");
        let expr   = iformat!("samplex({norm},{t.gradient.glsl()})");
        expr.into()
    }
}}

//
//impls! {[C:Mix+Clone] From<Gradient<C>> for Glsl {
//    |t| t.0
//}}


