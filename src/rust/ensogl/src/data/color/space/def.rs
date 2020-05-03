//! This module contains definitions of various color spaces, including `Rgb`, `Hsl`, `Lch`, etc.

use super::super::data::*;
use super::super::component::*;



// ==============
// === Macros ===
// ==============

macro_rules! replace {
    ($a:tt,$b:tt) => {$b}
}

macro_rules! define_color_space {
    ($(#[$($meta:tt)*])* $name:ident $a_name:ident $data_name:ident [$($comp:ident)*]) => {
        $(#[$($meta)*])*
        pub type $name = Color<$data_name>;

        $(#[$($meta)*])*
        pub type $a_name = Color<Alpha<$data_name>>;

        $(#[$($meta)*])*
        #[derive(Clone,Copy,Debug,PartialEq)]
        #[allow(missing_docs)]
        pub struct $data_name {
            $(pub $comp : f32),*
        }

        impl $data_name {
            /// Constructor.
            pub fn new($($comp:f32),*) -> Self {
                Self {$($comp),*}
            }
        }

        impl $name {
            /// Constructor.
            pub fn new($($comp:f32),*) -> Self {
                let data = $data_name::new($($comp),*);
                Self {data}
            }
        }

        impl $a_name {
            /// Constructor.
            pub fn new($($comp:f32),*,alpha:f32) -> Self {
                let color = $data_name::new($($comp),*);
                let data  = Alpha {alpha,color};
                Self {data}
            }
        }

        impl HasComponentsRepr for $data_name{
            type ComponentsRepr = ($(replace!($comp,f32)),*,);
        }

        impl From<$data_name> for ComponentsOf<$data_name> {
            fn from(data:$data_name) -> Self {
                Components(($(data.$comp.clone()),*,))
            }
        }

        impl From<ComponentsOf<$data_name>> for $data_name {
            fn from(Components(($($comp),*,)):ComponentsOf<Self>) -> Self {
                Self {$($comp),*}
            }
        }

        impl ComponentMap for $data_name {
            fn map<F:Fn(f32)->f32>(&self, f:F) -> Self {
                $(let $comp = f(self.$comp);)*
                Self {$($comp),*}
            }
        }
    };
}



// ===========
// === Rgb ===
// ===========

define_color_space! {
    /// The most common color space, when it comes to computer graphics, and it's defined as an
    /// additive mixture of red, green and blue light, where gray scale colors are created when
    /// these three channels are equal in strength.
    ///
    /// Many conversions and operations on this color space requires that it's linear, meaning that
    /// gamma correction is required when converting to and from a displayable `RGB` to `LinearRgb`.
    ///
    /// ## Parameters
    ///
    /// - `red` [0.0 - 1.0]
    ///   The amount of red light, where 0.0 is no red light and 1.0 is the highest displayable
    ///   amount.
    ///
    /// - `blue` [0.0 - 1.0]
    ///   The amount of blue light, where 0.0 is no blue light and 1.0 is the highest displayable
    ///   amount.
    ///
    /// - `green` [0.0 - 1.0]
    ///   The amount of green light, where 0.0 is no green light and 1.0 is the highest displayable
    ///   amount.
    Rgb Rgba RgbData [red green blue]
}

impl Rgb {
    /// Converts the color to `LinearRgb` representation.
    pub fn into_linear(self) -> LinearRgb {
        self.into()
    }
}

impl Rgba {
    /// Converts the color to `LinearRgba` representation.
    pub fn into_linear(self) -> LinearRgba {
        self.into()
    }
}



// =================
// === LinearRgb ===
// =================

define_color_space! {
    /// Linear sRGBv space. See `Rgb` to learn more.
    LinearRgb LinearRgba LinearRgbData [red green blue]
}



// ===========
// === Hsl ===
// ===========

define_color_space! {
    /// Linear HSL color space.
    ///
    /// The HSL color space can be seen as a cylindrical version of RGB, where the hue is the angle
    /// around the color cylinder, the saturation is the distance from the center, and the lightness
    /// is the height from the bottom. Its composition makes it especially good for operations like
    /// changing green to red, making a color more gray, or making it darker.
    ///
    /// See `Hsv` for a very similar color space, with brightness instead of lightness.
    ///
    /// ## Parameters
    ///
    /// - `hue` [0.0 - 1.0]
    ///   The hue of the color. Decides if it's red, blue, purple, etc. You can use `hue_degrees`
    ///   or `hue_radians` to gen hue in non-normalized form. Most implementations use value range
    ///   of [0 .. 360] instead. It was rescaled for convenience.
    ///
    /// - `saturation` [0.0 - 1.0]
    ///   The colorfulness of the color. 0.0 gives gray scale colors and 1.0 will give absolutely
    ///   clear colors.
    ///
    /// - `lightness` [0.0 - 1.0]
    ///   Decides how light the color will look. 0.0 will be black, 0.5 will give a clear color,
    ///   and 1.0 will give white.
    Hsl Hsla HslData [hue saturation lightness]
}



// ===========
// === Xyz ===
// ===========

define_color_space! {
    /// The CIE 1931 XYZ color space.
    ///
    /// XYZ links the perceived colors to their wavelengths and simply makes it possible to describe
    /// the way we see colors as numbers. It's often used when converting from one color space to an
    /// other, and requires a standard illuminant and a standard observer to be defined.
    ///
    /// Conversions and operations on this color space depend on the defined white point. This
    /// implementation uses the `D65` white point by default.
    ///
    /// ## Parameters
    ///
    /// - `x` [0.0 - 0.95047] for the default `D65` white point.
    ///   Scale of what can be seen as a response curve for the cone cells in the human eye. Its
    ///   range depends on the white point.
    ///
    /// - `y` [0.0 - 1.0]
    ///   Luminance of the color, where 0.0 is black and 1.0 is white.
    ///
    /// - `z` [0.0 - 1.08883] for the default `D65` white point.
    ///   Scale of what can be seen as the blue stimulation. Its range depends on the white point.
    Xyz Xyza XyzData [x y z]
}



// ===========
// === Lab ===
// ===========

define_color_space! {
    /// The CIE L*a*b* (CIELAB) color space.
    ///
    /// CIE L*a*b* is a device independent color space which includes all perceivable colors. It's
    /// sometimes used to convert between other color spaces, because of its ability to represent
    /// all of their colors, and sometimes in color manipulation, because of its perceptual
    /// uniformity. This means that the perceptual difference between two colors is equal to their
    /// numerical difference.
    ///
    /// ## Parameters
    /// The parameters of L*a*b* are quite different, compared to many other color spaces, so
    /// manipulating them manually may be unintuitive.
    ///
    /// - `lightness` [0.0 - 1.0]
    ///   Lightness of 0.0 gives absolute black and 1.0 gives the brightest white. Most
    ///   implementations use value range of [0 .. 100] instead. It was rescaled for convenience.
    ///
    /// - `a` [-1.0 - 1.0]
    ///   a* goes from red at -1.0 to green at 1.0. Most implementations use value range of
    ///   [-128 .. 127] instead. It was rescaled for convenience.
    ///
    /// - `b` [-1.0 - 1.0]
    ///   b* goes from yellow at -1.0 to blue at 1.0. Most implementations use value range of
    ///   [-128 .. 127] instead. It was rescaled for convenience.
    Lab Laba LabData [lightness a b]
}

impl LabData {
    /// Computes the `hue` in degrees of the current color.
    pub fn hue(&self) -> Option<f32> {
        if self.a == 0.0 && self.b == 0.0 {
            None
        } else {
            let mut hue = self.b.atan2(self.a) * 180.0 / std::f32::consts::PI;
            if hue < 0.0 { hue += 360.0 }
            Some(hue)
        }
    }
}



// ===========
// === Lch ===
// ===========

define_color_space! {
    /// CIE L*C*h°, a polar version of CIE L*a*b*.
    ///
    /// L*C*h° shares its range and perceptual uniformity with L*a*b*, but it's a cylindrical color
    /// space, like HSL and HSV. This gives it the same ability to directly change the hue and
    /// colorfulness of a color, while preserving other visual aspects.
    ///
    /// ## Parameters
    ///
    /// - `lightness` [0.0 - 1.0]
    ///   Lightness of 0.0 gives absolute black and 100.0 gives the brightest white. Most
    ///   implementations use value range of [0 .. 100] instead. It was rescaled for convenience.
    ///
    /// - `chroma` [0.0 - 1.32]
    ///   The colorfulness of the color. It's similar to saturation. 0.0 gives gray scale colors,
    ///   and numbers around 128-181 gives fully saturated colors. The upper limit of 128 should
    ///   include the whole L*a*b* space and some more. You can use higher values to target `P3`,
    ///   `Rec.2020`, or even larger color spaces. Most implementations use value range of
    ///   [0 .. 132] instead. It was rescaled for convenience.
    ///
    /// - `hue` [0.0 - 1.0]
    ///   The hue of the color. Decides if it's red, blue, purple, etc. You can use `hue_degrees`
    ///   or `hue_radians` to gen hue in non-normalized form. Most implementations use value range
    ///   of [0 .. 360] instead. It was rescaled for convenience.
    Lch Lcha LchData [lightness chroma hue]
}
