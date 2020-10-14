//! Define  `Animatable` for colors. Note that we choose the Lab space as state space for
//! animations to get nicer transitions in terms of lightness/chroma/hue and avoid the
//! discontinuities of the polar coordinates of Lcha (i.e., a transition from hue 1 to 359 would go
//! through all hues instead of taking the shorter trip "backwards").

use super::*;

use crate::gui::component::Animatable;

use nalgebra::Vector4;



// ==================
// === Animatable ===
// ==================

impl Animatable<Lcha> for Lcha {
    type State = Vector4<f32>;

    fn from_state(state:Self::State) -> Self {
        Laba::from(state).into()
    }

    fn to_state(entity:Self) -> Self::State {
        Laba::from(entity).into()
    }
}

impl Animatable<Rgba> for Rgba {
    type State = Vector4<f32>;

    fn from_state(state:Self::State) -> Self {
        Laba::from(state).into()
    }

    fn to_state(entity:Self) -> Self::State {
        Laba::from(entity).into()
    }
}

impl Animatable<Laba> for Laba {
    type State = Vector4<f32>;

    fn from_state(state:Self::State) -> Self {
        state.into()
    }

    fn to_state(entity:Self) -> Self::State {
        entity.into()
    }
}
