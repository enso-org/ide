//! Color mixing utilities.

use crate::prelude::*;
use super::*;

use crate::data::mix;
use crate::data::mix::Mixable;



// =================
// === Color Mix ===
// =================

macro_rules! define_mix_impl_repr {
    ($tp:ty => $via_tp:ty [$repr:ident]) => {
        impl Mixable for $tp { type Repr = $repr; }

        impl From<$tp> for mix::Space<$tp> {
            fn from(value:$tp) -> mix::Space<$tp> {
                mix::Space::new(<$via_tp>::from(value).into())
            }
        }

        impl From<mix::Space<$tp>> for $tp {
            fn from(t:mix::Space<$tp>) -> Self {
                <$via_tp>::from(t.value).into()
            }
        }
    }
}

macro_rules! define_mix_impls {
    ($($tp:ident => $via_tp:ident;)*) => {$(
        define_mix_impl_repr! {$tp                      => $via_tp                      [Vector3]}
        define_mix_impl_repr! {Color<Alpha<Model<$tp>>> => Color<Alpha<Model<$via_tp>>> [Vector4]}
    )*}
}


// === Impls ===

define_mix_impls! {
    Lab => Lab;
    Lch => Lab;
    Rgb => LinearRgb;
}
