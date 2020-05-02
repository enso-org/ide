//! Generic color management implementation. Implements multiple color spaces, including `Rgb`,
//! `LinearRgb`, `Hsv`, `Hsl`, `Xyz`, `Lab`, `Lch`, and others. Provides conversion utilities and
//! many helpers. It is inspired by different libraries, including Rust Palette. We are not using
//! Palette here because it is buggy (https://github.com/Ogeon/palette/issues/187), uses bounds
//! on structs which makes the bound appear in places they should not, uses too strict bounds,
//! and does not provide many useful conversions. Moreover, this library is not so generic, uses
//! `f32` everywhere and is much simpler.


use crate::prelude::*;
use crate::math::algebra::*;



pub trait PushBack<T> {
    type Output;
    fn push_back(self,t:T) -> Self::Output;
}

impl<X> PushBack<X> for () {
    type Output = (X,);
    fn push_back(self,x:X) -> Self::Output { (x,) }
}

impl<X,T1> PushBack<X> for (T1,) {
    type Output = (T1,X);
    fn push_back(self,x:X) -> Self::Output { (self.0,x) }
}

impl<X,T1,T2> PushBack<X> for (T1,T2) {
    type Output = (T1,T2,X);
    fn push_back(self,x:X) -> Self::Output { (self.0,self.1,x) }
}

impl<X,T1,T2,T3> PushBack<X> for (T1,T2,T3) {
    type Output = (T1,T2,T3,X);
    fn push_back(self,x:X) -> Self::Output { (self.0,self.1,self.2,x) }
}


pub trait PopBack {
    type Last;
    type Init;
    fn pop_back(self) -> (Self::Last,Self::Init);
}

impl<T1> PopBack for (T1,) {
    type Last = T1;
    type Init = ();
    fn pop_back(self) -> (Self::Last,Self::Init) { (self.0,()) }
}

impl<T1,T2> PopBack for (T1,T2) {
    type Last = T2;
    type Init = (T1,);
    fn pop_back(self) -> (Self::Last,Self::Init) { (self.1,(self.0,)) }
}

impl<T1,T2,T3> PopBack for (T1,T2,T3) {
    type Last = T3;
    type Init = (T1,T2);
    fn pop_back(self) -> (Self::Last,Self::Init) { (self.2,(self.0,self.1)) }
}

impl<T1,T2,T3,T4> PopBack for (T1,T2,T3,T4) {
    type Last = T4;
    type Init = (T1,T2,T3);
    fn pop_back(self) -> (Self::Last,Self::Init) { (self.3,(self.0,self.1,self.2)) }
}

macro_rules! color_convert_via {
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
    ($name:ident $a_name:ident $data_name:ident [$($comp:ident)*]) => {
        pub type $name   = Color<$data_name>;
        pub type $a_name = Color<Alpha<$data_name>>;

        /// Color structure definition.
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

impl<T,X> PushBack<X> for Components<T>
where T:PushBack<X> {
    type Output = Components<<T as PushBack<X>>::Output>;
    fn push_back(self,t:X) -> Self::Output {
        Components(self.0.push_back(t))
    }
}

impl<T> PopBack for Components<T>
where T:PopBack {
    type Last = <T as PopBack>::Last;
    type Init = Components<<T as PopBack>::Init>;
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
where C:HasComponentsRepr, ComponentsReprOf<C>:PushBack<f32> {
    type ComponentsRepr = <ComponentsReprOf<C> as PushBack<f32>>::Output;
}

impl<C> From<Alpha<C>> for ComponentsOf<Alpha<C>>
where C:HasComponents, ComponentsReprOf<C>:PushBack<f32> {
    fn from(t:Alpha<C>) -> Self {
        t.color.into().push_back(t.alpha)
    }
}

impl<C> From<ComponentsOf<Alpha<C>>> for Alpha<C>
where C:HasComponents, ComponentsReprOf<C>:PushBack<f32>,
      <ComponentsReprOf<C> as PushBack<f32>>::Output : PopBack<Last=f32,Init=ComponentsReprOf<C>> {
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



// ===============
// === Structs ===
// ===============

define_color_repr!(Rgb       Rgba       RgbData       [red green blue]);
define_color_repr!(LinearRgb LinearRgba LinearRgbData [red green blue]);
define_color_repr!(Hsl       Hsla       HslData       [hue saturation luminance]);
define_color_repr!(Xyz       Xyza       XyzData       [x y z]);
define_color_repr!(Lab       Laba       LabData       [l a b]);
define_color_repr!(Lch       Lcha       LchData       [luminance chroma hue]);


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
            Some(self.b.atan2(self.a) * 180.0 / std::f32::consts::PI)
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
        let luminance = (max + min) / 2.0;
        if(max == min){
            let hue        = 0.0;
            let saturation = 0.0;
            Self {hue,saturation,luminance}
        } else {
            let spread     = max - min;
            let saturation = if luminance > 0.5 {
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
            Self {hue,saturation,luminance}
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



color_convert_via! { RgbData -> LinearRgbData -> XyzData }
color_convert_via! { XyzData -> LinearRgbData -> RgbData }



// ===================
// === Xyz <-> Lab ===
// ===================

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

        let l = (y * 116.0) - 16.0;
        let a = (x - y) * 500.0;
        let b = (y - z) * 200.0;

        Self {l,a,b}
    }
}}

color_conversion! {
impl From<LabData> for XyzData {
    fn from(color:LabData) -> Self {
        let y = (color.l + 16.0) / 116.0;
        let x = y + (color.a / 500.0);
        let z = y - (color.b / 200.0);

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
        let luminance = color.l;
        let chroma    = (color.a * color.a + color.b * color.b).sqrt();
        let hue       = color.hue().unwrap_or(0.0);
        Self {luminance,chroma,hue}
    }
}}



pub fn test() {
    let rgb : Rgb = from_components(Components((0.2,0.4,0.6)));
    let hsl = Hsl::from(rgb);
    let xyz = Xyz::from(rgb);
    println!("{:?}",hsl);
    println!("{:?}",xyz);
}


impl From<Lcha> for Rgba {
    fn from(t:Lcha) -> Self {
        todo!()
    }
}


impl From<Lcha> for LinearRgba {
    fn from(t:Color<Alpha<LchData>>) -> Self {
        todo!()
    }
}


impl From<Color<Alpha<RgbData>>> for Color<Alpha<LchData>> {
    fn from(t:Color<Alpha<RgbData>>) -> Self {
        todo!()
    }
}
