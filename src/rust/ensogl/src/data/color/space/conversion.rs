//! This module defines conversions between all defined color spaces. The color conversions
//! equations base on http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html.
//!
//! **WARNING**
//! Be extra careful when developing color conversion equations. Many equations were re-scaled to
//! make them more pleasant to work, however, the equations you will fnd will probably work on
//! different value ranges. Read documentation for each color space very carefully.

#![allow(clippy::unreadable_literal)]
#![allow(clippy::excessive_precision)]

use super::def::*;
use super::super::component::*;
use super::super::data::*;
use super::white_point::traits::*;
use super::white_point;



// ==============
// === Macros ===
// ==============

macro_rules! color_conversion {
    (
        $(#[$($meta:tt)*])*
        impl $([$($bounds:tt)*])? From<$src:ty> for $tgt:ty { $($toks:tt)* }
    ) => {
        $(#[$($meta)*])*
        impl $(<$($bounds)*>)? From<$src> for $tgt { $($toks)* }

        $(#[$($meta)*])*
        impl $(<$($bounds)*>)? From<Alpha<$src>> for Alpha<$tgt> {
             fn from(src:Alpha<$src>) -> Self {
                 let alpha  = src.alpha;
                 let opaque = src.opaque.into();
                 Self {alpha,opaque}
             }
        }

        $(#[$($meta)*])*
        impl $(<$($bounds)*>)? From<Color<$src>> for Color<$tgt> {
             fn from(src:Color<$src>) -> Self {
                 Self {data : src.data.into()}
             }
        }

        $(#[$($meta)*])*
        impl $(<$($bounds)*>)? From<Color<Alpha<$src>>> for Color<Alpha<$tgt>> {
             fn from(src:Color<Alpha<$src>>) -> Self {
                 Self {data : src.data.into()}
             }
        }
    }
}

macro_rules! color_convert_via {
    ($src:ident <-> $via:ident <-> $tgt:ident) => {
        color_convert_via! { $src -> $via -> $tgt }
        color_convert_via! { $tgt -> $via -> $src }
    };

    ($src:ident -> $via:ident -> $tgt:ident) => {
        impl From<$src> for $tgt {
            fn from(src:$src) -> Self {
                $via::from(src).into()
            }
        }

        impl From<Color<$src>> for Color<$tgt> {
            fn from(src:Color<$src>) -> Self {
                <Color<$via>>::from(src).into()
            }
        }

        impl From<Alpha<$src>> for Alpha<$tgt> {
            fn from(src:Alpha<$src>) -> Self {
                <Alpha<$via>>::from(src).into()
            }
        }

        impl From<Color<Alpha<$src>>> for Color<Alpha<$tgt>> {
            fn from(src:Color<Alpha<$src>>) -> Self {
                <Color<Alpha<$via>>>::from(src).into()
            }
        }
    }
}



// =========================
// === Rgb <-> LinearRgb ===
// =========================

fn into_linear(x:f32) -> f32 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

fn from_linear(x:f32) -> f32 {
    if x <= 0.0031308 {
        x * 12.92
    } else {
        x.powf(1.0/2.4) * 1.055 - 0.055
    }
}

color_conversion! {
impl From<RgbData> for LinearRgbData {
    fn from(rgb:RgbData) -> Self {
        from_components(rgb.map(into_linear).into())
    }
}}

color_conversion! {
impl From<LinearRgbData> for RgbData {
    fn from(rgb:LinearRgbData) -> Self {
        from_components(rgb.map(from_linear).into())
    }
}}



// ===================
// === Rgb <-> Hsl ===
// ===================

color_conversion! {
impl From<RgbData> for HslData {
    fn from(color:RgbData) -> Self {
        let min       = color.red.min(color.green).min(color.blue);
        let max       = color.red.max(color.green).max(color.blue);
        let lightness = (max + min) / 2.0;
        if (max - min).abs() < std::f32::EPSILON {
            let hue        = 0.0;
            let saturation = 0.0;
            Self {hue,saturation,lightness}
        } else {
            let spread     = max - min;
            let saturation = if lightness > 0.5 {
                spread / (2.0 - max - min)
            } else {
                spread / (max + min)
            };
            let red_dist = if color.green < color.blue { 6.0 } else { 0.0 };
            let mut hue  =
                if (max - color.red).abs() < std::f32::EPSILON {
                    (color.green - color.blue) / spread + red_dist
                } else if (max - color.green).abs() < std::f32::EPSILON {
                    (color.blue - color.red) / spread + 2.0
                } else {
                    (color.red - color.green) / spread + 4.0
                };
            hue /= 6.0;
            Self {hue,saturation,lightness}
        }
    }
}}



// ===================
// === Rgb <-> Xyz ===
// ===================

color_conversion! {
/// Assumed D65 white point.
/// http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
impl From<LinearRgbData> for XyzData {
    fn from(c:LinearRgbData) -> Self {
        let x = c.red * 0.4124564 + c.green * 0.3575761 + c.blue * 0.1804375;
        let y = c.red * 0.2126729 + c.green * 0.7151522 + c.blue * 0.0721750;
        let z = c.red * 0.0193339 + c.green * 0.1191920 + c.blue * 0.9503041;
        Self {x,y,z}
    }
}}

color_conversion! {
/// Assumed D65 white point.
/// http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
impl From<XyzData> for LinearRgbData {
    fn from(c:XyzData) -> Self {
        let red   = c.x *  3.2404542 + c.y * -1.5371385 + c.z * -0.4985314;
        let green = c.x * -0.9692660 + c.y *  1.8760108 + c.z *  0.0415560;
        let blue  = c.x *  0.0556434 + c.y * -0.2040259 + c.z *  1.0572252;
        Self {red,green,blue}
    }
}}



// ===================
// === Xyz <-> Lab ===
// ===================

impl LabData {
    /// Normalize the a* or b* value from range [-128 .. 127] to [-1 .. 1].
    fn normalize_a_b(t:f32) -> f32 {
        (2.0 * (t + 128.0) / 255.0) - 1.0
    }

    /// Denormalize the a* or b* value from range [-1 .. 1] to [-128 .. 127].
    fn denormalize_a_b(t:f32) -> f32 {
        (255.0 * (t + 1.0) / 2.0) - 128.0
    }
}

color_conversion! {
impl From<XyzData> for LabData {
    fn from(xyz:XyzData) -> Self {
        fn convert(c:f32) -> f32 {
            let epsilon : f32 = 6.0/29.0;
            let epsilon = epsilon.powi(3);
            let kappa   = 841.0 / 108.0;
            let delta   = 4.0   / 29.0;
            if c > epsilon { c.cbrt() } else { (kappa * c) + delta }
        }

        let xyz = Color(xyz) / white_point::D65::get_xyz();

        let x = convert(xyz.x);
        let y = convert(xyz.y);
        let z = convert(xyz.z);

        let lightness = ((y * 116.0) - 16.0)/100.0;
        let a         = Self::normalize_a_b((x - y) * 500.0);
        let b         = Self::normalize_a_b((y - z) * 200.0);

        Self {lightness,a,b}
    }
}}

color_conversion! {
impl From<LabData> for XyzData {
    fn from(color:LabData) -> Self {
        let a = LabData::denormalize_a_b(color.a);
        let b = LabData::denormalize_a_b(color.b);
        let y = (color.lightness * 100.0 + 16.0) / 116.0;
        let x = y + (a / 500.0);
        let z = y - (b / 200.0);

        fn convert(c:f32) -> f32 {
            let epsilon = 6.0   / 29.0;
            let kappa   = 108.0 / 841.0;
            let delta   = 4.0   / 29.0;
            if c > epsilon { c.powi(3) } else { (c - delta) * kappa }
        }

        (Color(Self::new(convert(x),convert(y),convert(z))) * white_point::D65::get_xyz()).data
    }
}}



// ===================
// === Lab <-> Lch ===
// ===================

impl LchData {
    /// Normalize the a* or b* value from range [0 .. 132] to [0 .. 1].
    fn normalize_chroma(t:f32) -> f32 {
        t / 132.0
    }

    /// Denormalize the a* or b* value from range [0 .. 1] to [0 .. 132].
    fn denormalize_chroma(t:f32) -> f32 {
        t * 132.0
    }
}

color_conversion! {
impl From<LabData> for LchData {
    fn from(color:LabData) -> Self {
        let a         = LabData::denormalize_a_b(color.a);
        let b         = LabData::denormalize_a_b(color.b);
        let lightness = color.lightness;
        let chroma    = LchData::normalize_chroma((a*a + b*b).sqrt());
        let hue       = color.hue().unwrap_or(0.0) / 360.0;
        Self {lightness,chroma,hue}
    }
}}

color_conversion! {
impl From<LchData> for LabData {
    fn from(color:LchData) -> Self {
        let lightness = color.lightness;
        let angle     = color.hue * 2.0 * std::f32::consts::PI;
        let chroma    = LchData::denormalize_chroma(color.chroma);
        let a         = Self::normalize_a_b(chroma.max(0.0) * angle.cos());
        let b         = Self::normalize_a_b(chroma.max(0.0) * angle.sin());
        Self {lightness,a,b}
    }
}}



// =========================
// === Trans-Conversions ===
// =========================

color_convert_via! { RgbData <-> LinearRgbData <-> XyzData }
color_convert_via! { RgbData <-> XyzData       <-> LabData }
color_convert_via! { RgbData <-> LabData       <-> LchData }

color_convert_via! { LinearRgbData <-> XyzData <-> LabData }
color_convert_via! { LinearRgbData <-> LabData <-> LchData }
