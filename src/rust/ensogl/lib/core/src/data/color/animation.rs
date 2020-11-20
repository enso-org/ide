//! Define `Animatable` for colors. Note that we choose the Lab space as state space for
//! animations to get nicer transitions in terms of lightness/chroma/hue and avoid the
//! discontinuities of the polar coordinates of Lcha (i.e., a transition from hue 1 to 359 would go
//! through all hues instead of taking the shorter trip "backwards").

use crate::prelude::*;
use super::*;

use enso_frp as frp;

use crate::display::shape::*;
use crate::gui::component::HasAnimationSpaceRepr;
use crate::gui::component::AnimationLinearSpace;
use crate::gui::component;



// =================
// === Constants ===
// =================

/// Used for linear space multiplication, so animation engine will provide smoother animations.
const NORM_COLOR_SCALING_FACTOR : f32 = 100.0;



// =======================
// === Animatable Lcha ===
// =======================

impl HasAnimationSpaceRepr for Lcha { type AnimationSpaceRepr = Vector4; }

impl From<Lcha> for AnimationLinearSpace<Vector4> {
    fn from(value:Lcha) -> AnimationLinearSpace<Vector4> {
        let value : Vector4 = Laba::from(value).into();
        let value = value.map(|t|t*NORM_COLOR_SCALING_FACTOR);
        AnimationLinearSpace { value }
    }
}
impl Into<Lcha> for AnimationLinearSpace<Vector4> {
    fn into(self) -> Lcha {
        let value = self.value.map(|t|t/NORM_COLOR_SCALING_FACTOR);
        Laba::from(value).into()
    }
}



// ======================
// === Animatable Lch ===
// ======================

impl HasAnimationSpaceRepr for Lch { type AnimationSpaceRepr = Vector3; }

impl From<Lch> for AnimationLinearSpace<Vector3> {
    fn from(value:Lch) -> AnimationLinearSpace<Vector3> {
        let value : Vector3 = Lab::from(value).into();
        let value = value.map(|t|t*NORM_COLOR_SCALING_FACTOR);
        AnimationLinearSpace { value }
    }
}

impl Into<Lch> for AnimationLinearSpace<Vector3> {
    fn into(self) -> Lch {
        let value = self.value.map(|t|t/NORM_COLOR_SCALING_FACTOR);
        Lab::from(value).into()
    }
}



// =======================
// === Animatable Rgba ===
// =======================

impl HasAnimationSpaceRepr for Rgba { type AnimationSpaceRepr = Vector4; }

impl From<Rgba> for AnimationLinearSpace<Vector4> {
    fn from(value:Rgba) -> AnimationLinearSpace<Vector4> {
        let value : Vector4 = Laba::from(value).into();
        let value = value.map(|t|t*NORM_COLOR_SCALING_FACTOR);
        AnimationLinearSpace { value }
    }
}

impl Into<Rgba> for AnimationLinearSpace<Vector4> {
    fn into(self) -> Rgba {
        let value = self.value.map(|t|t/NORM_COLOR_SCALING_FACTOR);
        Laba::from(value).into()
    }
}



// ======================
// === Animatable Rgb ===
// ======================

impl HasAnimationSpaceRepr for Rgb { type AnimationSpaceRepr = Vector3; }

impl From<Rgb> for AnimationLinearSpace<Vector3> {
    fn from(value:Rgb) -> AnimationLinearSpace<Vector3> {
        let value : Vector3 = Lab::from(value).into();
        let value = value.map(|t|t*NORM_COLOR_SCALING_FACTOR);
        AnimationLinearSpace { value }
    }
}

impl Into<Rgb> for AnimationLinearSpace<Vector3> {
    fn into(self) -> Rgb {
        let value = self.value.map(|t|t/NORM_COLOR_SCALING_FACTOR);
        Lab::from(value).into()
    }
}



// =======================
// === Animatable Laba ===
// =======================

impl HasAnimationSpaceRepr for Laba { type AnimationSpaceRepr = Vector4; }

impl From<Laba> for AnimationLinearSpace<Vector4> {
    fn from(value:Laba) -> AnimationLinearSpace<Vector4> {
        let value = value.into();
        AnimationLinearSpace { value }
    }
}

impl Into<Laba> for AnimationLinearSpace<Vector4> {
    fn into(self) -> Laba {
        self.value.into()
    }
}



// ======================
// === Animatable Lab ===
// ======================

impl HasAnimationSpaceRepr for Lab { type AnimationSpaceRepr = Vector3; }

impl From<Lab> for AnimationLinearSpace<Vector3> {
    fn from(value:Lab) -> AnimationLinearSpace<Vector3> {
        let value = value.into();
        AnimationLinearSpace { value }
    }
}

impl Into<Lab> for AnimationLinearSpace<Vector3> {
    fn into(self) -> Lab {
        self.value.into()
    }
}



// =======================
// === Color Animation ===
// =======================

crate::define_endpoints! {
    Input {
        target       (Lcha),
        target_alpha (f32),
        target_color (Lch),
    }
    Output {
        value (Lcha),
    }
}

/// The `Animation` provides color better animations for colors than the raw
/// `component::DEPRECATED_Animation<_>`, as it allows controlling the alpha channel separately which is
/// important for nice fade outs.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Animation {
    frp        : FrpEndpoints,
    color_anim : component::Animation<Lch>,
    alpha_anim : component::Animation<f32>,
}

impl Deref for Animation {
    type Target = FrpEndpoints;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl Animation {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        let frp        = Frp::extend(network);
        let color_anim = component::Animation::new_non_init(network);
        let alpha_anim = component::Animation::new_non_init(network);
        Self{frp,color_anim,alpha_anim}.init(network)
    }

    fn init(self, network:&frp::Network) -> Self {
        frp::extend! { network
            color_of_target        <- self.frp.target.map(|t|t.opaque);
            alpha_of_target        <- self.frp.target.map(|t|t.alpha);
            target_color           <- any(&self.frp.target_color,&color_of_target);
            target_alpha           <- any(&self.frp.target_alpha,&alpha_of_target);
            self.color_anim.target <+ target_color;
            self.alpha_anim.target <+ target_alpha;
            self.frp.source.value  <+ all(&self.color_anim.value,&self.alpha_anim.value).map(
                |(color,alpha)| color.with_alpha(*alpha)
            );
        }
        self
    }
}
