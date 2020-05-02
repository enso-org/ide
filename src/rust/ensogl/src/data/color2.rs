//! Generic color management implementation. Implements multiple color spaces, including `Rgb`,
//! `LinearRgb`, `Hsv`, `Hsl`, `Xyz`, `Lab`, `Lch`, and others. Provides conversion utilities and
//! many helpers. It is inspired by different libraries, including Rust Palette. We are not using
//! Palette here because it is buggy (https://github.com/Ogeon/palette/issues/187), uses bounds
//! on structs which makes the bound appear in places they should not, uses too strict bounds,
//! and does not provide many useful conversions. Moreover, this library is not so generic, uses
//! `f32` everywhere and is much simpler.
//!
//! **WARNING**
//! Be extra careful when developing color conversion equations. Many equations were re-scaled to
//! make them more pleasant to work, however, the equations you will fnd will probably work on
//! different value ranges. Read documentation for each color space very carefully.

use crate::prelude::*;
use crate::math::algebra::*;


use enso_generics::*;
use enso_generics as generic;
use enso_generics as hlist;










//
//impl generic::PushBack<X> for T {
//    type Output =
//}









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


macro_rules! replace {
    ($a:tt,$b:tt) => {$b}
}

macro_rules! define_color_repr {
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


// ==================
// === WhitePoint ===
// ==================

/// Xyz color co-ordinates for a given white point.
///
/// A white point (often referred to as reference white or target white in technical documents)
/// is a set of tristimulus values or chromaticity coordinates that serve to define the color
/// "white" in image capture, encoding, or reproduction.
///
/// Custom white points can be easily defined on an empty struct with the tristimulus values
/// and can be used in place of the ones defined in this library.
pub trait WhitePoint {
    ///Get the Xyz chromacity co-ordinates for the white point.
    fn get_xyz() -> Xyz;
}

/// CIE D series standard illuminant - D65.
///
/// D65 White Point is the natural daylight with a color temperature of 6500K for 2° Standard
/// Observer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct D65;
impl WhitePoint for D65 {
    fn get_xyz() -> Xyz {
        from_components(Components((0.95047,1.0,1.08883)))
    }
}



// =================
// === Component ===
// =================


pub trait HasComponentsRepr {
    type ComponentsRepr;
}

pub type ComponentsReprOf<T> = <T as HasComponentsRepr>::ComponentsRepr;


pub trait ComponentMap {
    fn map<F:Fn(f32)->f32>(&self, f:F) -> Self;
}

pub type ComponentsOf<T> = Components<ComponentsReprOf<T>>;

#[derive(Clone,Copy,Debug)]
pub struct Components<T>(T);

pub trait ToComponents   = Sized + HasComponentsRepr + Into<ComponentsOf<Self>>;
pub trait FromComponents = Sized + HasComponentsRepr where ComponentsOf<Self> : Into<Self>;
pub trait HasComponents : ToComponents + FromComponents {
    fn from_components(components:ComponentsOf<Self>) -> Self {
        components.into()
    }
    fn into_components(self) -> ComponentsOf<Self> {
        self.into()
    }
}
impl<T> HasComponents for T where T : ToComponents + FromComponents {}



pub fn from_components<T:FromComponents>(components:ComponentsOf<T>) -> T {
    components.into()
}

impl<T,X> generic::PushBack<X> for Components<T>
where T:generic::PushBack<X> {
    type Output = Components<<T as generic::PushBack<X>>::Output>;
    fn push_back(self,t:X) -> Self::Output {
        Components(self.0.push_back(t))
    }
}

impl<T:KnownLast> KnownLast for Components<T> { type Last = Last<T>; }
impl<T:KnownInit> KnownInit for Components<T> { type Init = Components<Init<T>>; }


impl<T> PopBack for Components<T>
where T:PopBack {
    fn pop_back(self) -> (Self::Last,Self::Init) {
        let (last,init) = self.0.pop_back();
        let init = Components(init);
        (last,init)
    }
}


macro_rules! components_opr {
    ($($toks:tt)*) => {
        components_opr_3! { $($toks)* }
        components_opr_4! { $($toks)* }
    }
}

macro_rules! components_opr_3 {
    ($($name:ident :: $fn:ident),*) => {$(
        impl $name<f32> for Components<(f32,f32,f32)> {
            type Output = Components<(f32,f32,f32)>;
            fn $fn(self, r:f32) -> Self::Output {
                let t = self.0;
                Components(((t.0).$fn(r), (t.1).$fn(r), (t.2).$fn(r)))
            }
        }

        impl $name<&f32> for Components<(f32,f32,f32)> {
            type Output = Components<(f32,f32,f32)>;
            fn $fn(self, r:&f32) -> Self::Output {
                self.$fn(*r)
            }
        }

        impl $name<f32> for &Components<(f32,f32,f32)> {
            type Output = Components<(f32,f32,f32)>;
            fn $fn(self, r:f32) -> Self::Output {
                (*self).$fn(r)
            }
        }

        impl $name<&f32> for &Components<(f32,f32,f32)> {
            type Output = Components<(f32,f32,f32)>;
            fn $fn(self, r:&f32) -> Self::Output {
                (*self).$fn(*r)
            }
        }

        impl $name<Components<(f32,f32,f32)>> for Components<(f32,f32,f32)> {
            type Output = Components<(f32,f32,f32)>;
            fn $fn(self, r:Components<(f32,f32,f32)>) -> Self::Output {
                let t = self.0;
                let r = r.0;
                Components(((t.0).$fn(r.0), (t.1).$fn(r.1), (t.2).$fn(r.2)))
            }
        }

        impl $name<&Components<(f32,f32,f32)>> for Components<(f32,f32,f32)> {
            type Output = Components<(f32,f32,f32)>;
            fn $fn(self, r:&Components<(f32,f32,f32)>) -> Self::Output {
                self.$fn(*r)
            }
        }

        impl $name<Components<(f32,f32,f32)>> for &Components<(f32,f32,f32)> {
            type Output = Components<(f32,f32,f32)>;
            fn $fn(self, r:Components<(f32,f32,f32)>) -> Self::Output {
                (*self).$fn(r)
            }
        }

        impl $name<&Components<(f32,f32,f32)>> for &Components<(f32,f32,f32)> {
            type Output = Components<(f32,f32,f32)>;
            fn $fn(self, r:&Components<(f32,f32,f32)>) -> Self::Output {
                (*self).$fn(*r)
            }
        }
    )*}
}

macro_rules! components_opr_4 {
    ($($name:ident :: $fn:ident),*) => {$(
        impl $name<f32> for Components<(f32,f32,f32,f32)> {
            type Output = Components<(f32,f32,f32,f32)>;
            fn $fn(self, r:f32) -> Self::Output {
                let t = self.0;
                Components(((t.0).$fn(r), (t.1).$fn(r), (t.2).$fn(r), (t.3).$fn(r)))
            }
        }

        impl $name<&f32> for Components<(f32,f32,f32,f32)> {
            type Output = Components<(f32,f32,f32,f32)>;
            fn $fn(self, r:&f32) -> Self::Output {
                self.$fn(*r)
            }
        }

        impl $name<f32> for &Components<(f32,f32,f32,f32)> {
            type Output = Components<(f32,f32,f32,f32)>;
            fn $fn(self, r:f32) -> Self::Output {
                (*self).$fn(r)
            }
        }

        impl $name<&f32> for &Components<(f32,f32,f32,f32)> {
            type Output = Components<(f32,f32,f32,f32)>;
            fn $fn(self, r:&f32) -> Self::Output {
                (*self).$fn(*r)
            }
        }

        impl $name<Components<(f32,f32,f32,f32)>> for Components<(f32,f32,f32,f32)> {
            type Output = Components<(f32,f32,f32,f32)>;
            fn $fn(self, r:Components<(f32,f32,f32,f32)>) -> Self::Output {
                let t = self.0;
                let r = r.0;
                Components(((t.0).$fn(r.0), (t.1).$fn(r.1), (t.2).$fn(r.2), (t.3).$fn(r.3)))
            }
        }

        impl $name<&Components<(f32,f32,f32,f32)>> for Components<(f32,f32,f32,f32)> {
            type Output = Components<(f32,f32,f32,f32)>;
            fn $fn(self, r:&Components<(f32,f32,f32,f32)>) -> Self::Output {
                self.$fn(*r)
            }
        }

        impl $name<Components<(f32,f32,f32,f32)>> for &Components<(f32,f32,f32,f32)> {
            type Output = Components<(f32,f32,f32,f32)>;
            fn $fn(self, r:Components<(f32,f32,f32,f32)>) -> Self::Output {
                (*self).$fn(r)
            }
        }

        impl $name<&Components<(f32,f32,f32,f32)>> for &Components<(f32,f32,f32,f32)> {
            type Output = Components<(f32,f32,f32,f32)>;
            fn $fn(self, r:&Components<(f32,f32,f32,f32)>) -> Self::Output {
                (*self).$fn(*r)
            }
        }
    )*}
}

components_opr! { Add::add, Sub::sub, Mul::mul, Div::div }





// =============
// === Color ===
// =============

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Color<D> {
    data : D
}

pub fn Color<D>(data:D) -> Color<D> {
    Color {data}
}

impl<D:ComponentMap> ComponentMap for Color<D> {
    fn map<F:Fn(f32)->f32>(&self, f:F) -> Self {
        Self {data:self.data.map(f)}
    }
}

impl<D:HasComponentsRepr> HasComponentsRepr for Color<D> {
    type ComponentsRepr = ComponentsReprOf<D>;
}

impl<D> From<Color<D>> for ComponentsOf<Color<D>>
where D:HasComponents {
    fn from(color:Color<D>) -> Self {
        color.data.into()
    }
}

impl<D> From<ComponentsOf<D>> for Color<D>
where D:HasComponentsRepr, ComponentsOf<D>:Into<D> {
    fn from(components:ComponentsOf<D>) -> Self {
        Self {data:components.into()}
    }
}

impl<D> Deref for Color<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<D1,D2> From<&Color<D1>> for Color<D2>
where Color<D1> : Clone + Into<Color<D2>> {
    fn from(color:&Color<D1>) -> Self {
        color.clone().into()
    }
}

impl<C> From<Color<C>> for Color<Alpha<C>> {
    fn from(color:Color<C>) -> Self {
        let data = color.data.into();
        Self {data}
    }
}


macro_rules! color_opr {
    ($($name:ident :: $fn:ident),*) => {$(
        impl<D> $name<f32> for Color<D>
        where Self : HasComponents,
              ComponentsOf<Self> : $name<f32>,
              <ComponentsOf<Self> as $name<f32>>::Output : Into<Self> {
            type Output = Self;
            fn $fn(self, rhs:f32) -> Self::Output {
                self.into_components().$fn(rhs).into()
            }
        }

        impl<D> $name<&f32> for Color<D>
        where Color<D> : $name<f32> {
            type Output = <Color<D> as $name<f32>>::Output;
            fn $fn(self, rhs:&f32) -> Self::Output {
                self.$fn(*rhs)
            }
        }

        impl<D> $name<f32> for &Color<D>
        where Color<D> : Copy + $name<f32> {
            type Output = <Color<D> as $name<f32>>::Output;
            fn $fn(self, rhs:f32) -> Self::Output {
                (*self).$fn(rhs)
            }
        }

        impl<D> $name<&f32> for &Color<D>
        where Color<D> : Copy + $name<f32> {
            type Output = <Color<D> as $name<f32>>::Output;
            fn $fn(self, rhs:&f32) -> Self::Output {
                (*self).$fn(*rhs)
            }
        }

        impl<D> $name<Color<D>> for Color<D>
        where Color<D> : HasComponents,
              ComponentsOf<Color<D>> : $name<ComponentsOf<Color<D>>>,
              <ComponentsOf<Color<D>> as $name<ComponentsOf<Color<D>>>>::Output : Into<Self> {
            type Output = Self;
            fn $fn(self, rhs:Color<D>) -> Self::Output {
                self.into_components().$fn(rhs.into_components()).into()
            }
        }

    )*}
}

color_opr!{ Add::add, Sub::sub, Mul::mul, Div::div }



// =============
// === Alpha ===
// =============

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Alpha<C> {
    pub alpha : f32,
    pub color : C,
}


impl<C> HasComponentsRepr for Alpha<C>
where C:HasComponentsRepr, ComponentsReprOf<C>:generic::PushBack<f32> {
    type ComponentsRepr = <ComponentsReprOf<C> as generic::PushBack<f32>>::Output;
}

impl<C> From<Alpha<C>> for ComponentsOf<Alpha<C>>
where C:HasComponents, ComponentsReprOf<C>:generic::PushBack<f32> {
    fn from(t:Alpha<C>) -> Self {
        t.color.into().push_back(t.alpha)
    }
}

impl<C> From<ComponentsOf<Alpha<C>>> for Alpha<C>
where C:HasComponents, ComponentsReprOf<C>:generic::PushBack<f32>,
      <ComponentsReprOf<C> as generic::PushBack<f32>>::Output : PopBack<Last=f32,Init=ComponentsReprOf<C>> {
    fn from(components:ComponentsOf<Self>) -> Self {
        let (alpha,init) = components.pop_back();
        let color        = from_components(init);
        Self {alpha,color}
    }
}

impl<C:ComponentMap> ComponentMap for Alpha<C> {
    fn map<F:Fn(f32)->f32>(&self, f:F) -> Self {
        let alpha = f(self.alpha);
        let color = self.color.map(f);
        Self {alpha,color}
    }
}

impl<C> Deref for Alpha<C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        &self.color
    }
}

impl<C> From<C> for Alpha<C> {
    fn from(color:C) -> Self {
        let alpha = 1.0;
        Self {alpha,color}
    }
}



// ====================
// === Color Spaces ===
// ====================

define_color_repr! {
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

define_color_repr! {
    /// Linear sRGBv space. See `Rgb` to learn more.
    LinearRgb LinearRgba LinearRgbData [red green blue]
}

define_color_repr! {
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

define_color_repr! {
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

define_color_repr! {
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

define_color_repr! {
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


impl Rgba {
    pub fn into_linear(self) -> LinearRgba {
        self.into()
    }
}

impl LabData {
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

macro_rules! color_conversion {
    (
        impl $([$($bounds:tt)*])? From<$src:ty> for $tgt:ty { $($toks:tt)* }
    ) => {
        impl $(<$($bounds)*>)? From<$src> for $tgt { $($toks)* }

        impl $(<$($bounds)*>)? From<Alpha<$src>> for Alpha<$tgt> {
             fn from(src:Alpha<$src>) -> Self {
                 let alpha = src.alpha;
                 let color = src.color.into();
                 Self {alpha,color}
             }
        }

        impl $(<$($bounds)*>)? From<Color<$src>> for Color<$tgt> {
             fn from(src:Color<$src>) -> Self {
                 Self {data : src.data.into()}
             }
        }

        impl $(<$($bounds)*>)? From<Color<Alpha<$src>>> for Color<Alpha<$tgt>> {
             fn from(src:Color<Alpha<$src>>) -> Self {
                 Self {data : src.data.into()}
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
        from_components(rgb.map(|t| into_linear(t)).into())
    }
}}

color_conversion! {
impl From<LinearRgbData> for RgbData {
    fn from(rgb:LinearRgbData) -> Self {
        from_components(rgb.map(|t| from_linear(t)).into())
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
        if(max == min){
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
            let red_dist = if (color.green < color.blue) { 6.0 } else { 0.0 };
            let mut hue  =
                if      color.red   == max { (color.green - color.blue)  / spread + red_dist }
                else if color.green == max { (color.blue  - color.red)   / spread + 2.0 }
                else                       { (color.red   - color.green) / spread + 4.0 };
            hue = hue / 6.0;
            Self {hue,saturation,lightness}
        }
    }
}}



// ===================
// === Rgb <-> Xyz ===
// ===================

/// Assumed D65 white point.
/// http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
color_conversion! {
impl From<LinearRgbData> for XyzData {
    fn from(c:LinearRgbData) -> Self {
        let x = c.red * 0.4124564 + c.green * 0.3575761 + c.blue * 0.1804375;
        let y = c.red * 0.2126729 + c.green * 0.7151522 + c.blue * 0.0721750;
        let z = c.red * 0.0193339 + c.green * 0.1191920 + c.blue * 0.9503041;
        Self {x,y,z}
    }
}}

color_conversion! {
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

        let xyz = Color(xyz) / D65::get_xyz();

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

        (Color(Self::new(convert(x),convert(y),convert(z))) * D65::get_xyz()).data
    }
}}


// ===================
// === Lab <-> Lch ===
// ===================

color_conversion! {
impl From<LabData> for LchData {
    fn from(color:LabData) -> Self {
        let a         = LabData::denormalize_a_b(color.a);
        let b         = LabData::denormalize_a_b(color.b);
        let lightness = color.lightness;
        let chroma    = (a*a + b*b).sqrt();
        let hue       = color.hue().unwrap_or(0.0) / 360.0;
        Self {lightness,chroma,hue}
    }
}}

color_conversion! {
impl From<LchData> for LabData {
    fn from(color:LchData) -> Self {
        let lightness = color.lightness;
        let angle     = color.hue * 2.0 * std::f32::consts::PI;
        let a         = Self::normalize_a_b(color.chroma.max(0.0) * angle.cos());
        let b         = Self::normalize_a_b(color.chroma.max(0.0) * angle.sin());
        Self {lightness,a,b}
    }
}}




color_convert_via! { RgbData <-> LinearRgbData <-> XyzData }
color_convert_via! { RgbData <-> XyzData       <-> LabData }
color_convert_via! { RgbData <-> LabData       <-> LchData }


color_convert_via! { LinearRgbData <-> XyzData <-> LabData }
color_convert_via! { LinearRgbData <-> LabData <-> LchData }




pub fn test() {
    let rgb : Rgb = from_components(Components((0.2,0.4,0.6)));
    let hsl = Hsl::from(rgb);
    let xyz = Xyz::from(rgb);
    println!("{:?}",hsl);
    println!("{:?}",xyz);
}
