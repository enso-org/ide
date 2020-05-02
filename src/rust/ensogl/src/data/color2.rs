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

macro_rules! convert_via {
    ($src:ident -> $via:ident -> $tgt:ident) => {
        impl From<$src> for $tgt {
            fn from(src:$src) -> Self {
                $via::from(src).into()
            }
        }
    }
}


macro_rules! replace {
    ($a:tt,$b:tt) => {$b}
}

macro_rules! define_color_repr {
    ($name:ident $a_name:ident $data_name:ident [$($comp:ident)*]) => {
        pub type $name   <T=f32> = Color<$data_name<T>>;
        pub type $a_name <T=f32> = Color<Alpha<$data_name<T>>>;

        #[derive(Clone,Copy,Debug)]
        pub struct $data_name<T=f32> {
            $($comp : T),*
        }

        impl<T:Clone> $data_name<T> {
            $(pub fn $comp(&self) -> T {
                self.$comp.clone()
            })*
        }

        impl<T> HasComponent for $data_name<T> {
            type Component = T;
        }

        impl<T> HasComponentsRepr for $data_name<T>{
            type ComponentsRepr = ($(replace!($comp,T)),*,);
        }

        impl<T:Clone> ToComponents for $data_name<T> {
            fn to_components(&self) -> Self::ComponentsRepr {
                ($(self.$comp.clone()),*,)
            }
        }

        impl<T> FromComponents for $data_name<T> {
            fn from_components(($($comp),*,):Self::ComponentsRepr) -> Self {
                Self {$($comp),*}
            }
        }

        impl<T> ComponentMap for $data_name<T> {
            fn map<F:Fn(&ComponentOf<Self>)->ComponentOf<Self>>(&self, f:F) -> Self {
                $(let $comp = f(&self.$comp);)*
                Self {$($comp),*}
            }
        }

        impl<T:Component> Div<$data_name<T>> for $data_name<T> {
            type Output = $data_name<T>;
            fn div(self, rhs:$data_name<T>) -> Self::Output {
                $(let $comp = self.$comp / rhs.$comp;)*
                Self {$($comp),*}
            }
        }

        impl<T:Component> Div<&$data_name<T>> for $data_name<T> {
            type Output = $data_name<T>;
            fn div(self, rhs:&$data_name<T>) -> Self::Output {
                self / rhs.clone()
            }
        }

        impl<T:Component> Div<$data_name<T>> for &$data_name<T> {
            type Output = $data_name<T>;
            fn div(self, rhs:$data_name<T>) -> Self::Output {
                self.clone() / rhs
            }
        }

        impl<T:Component> Div<&$data_name<T>> for &$data_name<T> {
            type Output = $data_name<T>;
            fn div(self, rhs:&$data_name<T>) -> Self::Output {
                self.clone() / rhs.clone()
            }
        }

        impl<T:Component> Mul<$data_name<T>> for $data_name<T> {
            type Output = $data_name<T>;
            fn mul(self, rhs:$data_name<T>) -> Self::Output {
                $(let $comp = self.$comp * rhs.$comp;)*
                Self {$($comp),*}
            }
        }

        impl<T:Component> Mul<&$data_name<T>> for $data_name<T> {
            type Output = $data_name<T>;
            fn mul(self, rhs:&$data_name<T>) -> Self::Output {
                self * rhs.clone()
            }
        }

        impl<T:Component> Mul<$data_name<T>> for &$data_name<T> {
            type Output = $data_name<T>;
            fn mul(self, rhs:$data_name<T>) -> Self::Output {
                self.clone() * rhs
            }
        }

        impl<T:Component> Mul<&$data_name<T>> for &$data_name<T> {
            type Output = $data_name<T>;
            fn mul(self, rhs:&$data_name<T>) -> Self::Output {
                self.clone() * rhs.clone()
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
    fn get_xyz<T:Component>() -> Xyz<T>;
}

/// CIE D series standard illuminant - D65.
///
/// D65 White Point is the natural daylight with a color temperature of 6500K for 2° Standard
/// Observer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct D65;
impl WhitePoint for D65 {
    fn get_xyz<T:Component>() -> Xyz<T> {
        from_components((0.95047.into(), 1.0.into(), 1.08883.into()))
    }
}



// =================
// === Component ===
// =================

alias2! { Component
    = Clone
    + From<f32>
    + Add<Output=Self>
    + Sub<Output=Self>
    + Mul<Output=Self>
    + Div<Output=Self>
    + Pow<Output=Self>
    + Signum<Output=Self>
    + Clamp<Output=Self>
}

pub trait HasComponent {
    type Component;
}

pub trait HasComponentsRepr {
    type ComponentsRepr;
}

pub type ComponentOf<T> = <T as HasComponent>::Component;
pub type ComponentsReprOf<T> = <T as HasComponentsRepr>::ComponentsRepr;


pub trait ComponentMap : HasComponent {
    fn map<F:Fn(&ComponentOf<Self>)->ComponentOf<Self>>(&self, f:F) -> Self;
}

pub trait ToComponents : HasComponentsRepr {
    fn to_components(&self) -> Self::ComponentsRepr;
}

pub trait FromComponents : HasComponentsRepr {
    fn from_components(repr:Self::ComponentsRepr) -> Self;
}

pub fn from_components<T:FromComponents>(repr:ComponentsReprOf<T>) -> T {
    T::from_components(repr)
}



// =============
// === Color ===
// =============

#[derive(Clone,Copy,Debug)]
pub struct Color<D> {
    data : D
}

impl<D:HasComponent> HasComponent for Color<D> {
    type Component = ComponentOf<D>;
}

impl<D:ComponentMap> ComponentMap for Color<D> {
    fn map<F:Fn(&ComponentOf<Self>)->ComponentOf<Self>>(&self, f:F) -> Self {
        Self {data:self.data.map(f)}
    }
}

impl<D:HasComponentsRepr> HasComponentsRepr for Color<D> {
    type ComponentsRepr = ComponentsReprOf<D>;
}

impl<D:ToComponents> ToComponents for Color<D> {
    fn to_components(&self) -> Self::ComponentsRepr {
        self.data.to_components()
    }
}

impl<D:FromComponents> FromComponents for Color<D> {
    fn from_components(repr:Self::ComponentsRepr) -> Self {
        Self {data:from_components(repr)}
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

impl<D1,D2> Div<Color<D2>> for Color<D1>
where D1:Div<D2> {
    type Output = Color<<D1 as Div<D2>>::Output>;
    fn div(self, rhs:Color<D2>) -> Color<<D1 as Div<D2>>::Output> {
        Color { data : self.data / rhs.data }
    }
}

impl<D1,D2> Div<&Color<D2>> for Color<D1>
where D1:Div<D2>, D2:Clone {
    type Output = Color<<D1 as Div<D2>>::Output>;
    fn div(self, rhs:&Color<D2>) -> Color<<D1 as Div<D2>>::Output> {
        self / rhs.clone()
    }
}

impl<D1,D2> Div<Color<D2>> for &Color<D1>
where D1:Div<D2>, D1:Clone {
    type Output = Color<<D1 as Div<D2>>::Output>;
    fn div(self, rhs:Color<D2>) -> Color<<D1 as Div<D2>>::Output> {
        self.clone() / rhs
    }
}

impl<D1,D2> Div<&Color<D2>> for &Color<D1>
where D1:Div<D2>, D1:Clone, D2:Clone {
    type Output = Color<<D1 as Div<D2>>::Output>;
    fn div(self, rhs:&Color<D2>) -> Color<<D1 as Div<D2>>::Output> {
        self.clone() / rhs.clone()
    }
}



impl<D1,D2> Mul<Color<D2>> for Color<D1>
where D1:Mul<D2> {
    type Output = Color<<D1 as Mul<D2>>::Output>;
    fn mul(self, rhs:Color<D2>) -> Color<<D1 as Mul<D2>>::Output> {
        Color { data : self.data * rhs.data }
    }
}

impl<D1,D2> Mul<&Color<D2>> for Color<D1>
where D1:Mul<D2>, D2:Clone {
    type Output = Color<<D1 as Mul<D2>>::Output>;
    fn mul(self, rhs:&Color<D2>) -> Color<<D1 as Mul<D2>>::Output> {
        self * rhs.clone()
    }
}

impl<D1,D2> Mul<Color<D2>> for &Color<D1>
where D1:Mul<D2>, D1:Clone {
    type Output = Color<<D1 as Mul<D2>>::Output>;
    fn mul(self, rhs:Color<D2>) -> Color<<D1 as Mul<D2>>::Output> {
        self.clone() * rhs
    }
}

impl<D1,D2> Mul<&Color<D2>> for &Color<D1>
where D1:Mul<D2>, D1:Clone, D2:Clone {
    type Output = Color<<D1 as Mul<D2>>::Output>;
    fn mul(self, rhs:&Color<D2>) -> Color<<D1 as Mul<D2>>::Output> {
        self.clone() * rhs.clone()
    }
}


// =============
// === Alpha ===
// =============

pub type Alpha<C> = AlphaData<ComponentOf<C>,C>;

#[derive(Clone,Copy,Debug)]
pub struct AlphaData<A,C> {
    alpha : A,
    color : C,
}

impl<A,C:HasComponent> HasComponent for AlphaData<A,C> {
    type Component = ComponentOf<C>;
}

impl<A,C> HasComponentsRepr for AlphaData<A,C>
where C:ToComponents, ComponentsReprOf<C>:PushBack<A> {
    type ComponentsRepr = <ComponentsReprOf<C> as PushBack<A>>::Output;
}

impl<A,C> ToComponents for AlphaData<A,C>
where A:Clone, C:ToComponents, ComponentsReprOf<C>:PushBack<A> {
    fn to_components(&self) -> Self::ComponentsRepr {
        self.color.to_components().push_back(self.alpha.clone())
    }
}

impl<A,C> FromComponents for AlphaData<A,C>
where C:FromComponents + ToComponents,
      ComponentsReprOf<C>:PushBack<A>,
      <ComponentsReprOf<C> as PushBack<A>>::Output : PopBack<Last=A,Init=ComponentsReprOf<C>> {
    fn from_components(repr:Self::ComponentsRepr) -> Self {
        let (alpha,init) = repr.pop_back();
        let color        = from_components(init);
        Self {alpha,color}
    }
}

impl<C:HasComponent + ComponentMap> ComponentMap for Alpha<C> {
    fn map<F:Fn(&ComponentOf<Self>)->ComponentOf<Self>>(&self, f:F) -> Self {
        let alpha = f(&self.alpha);
        let color = self.color.map(f);
        Self {alpha,color}
    }
}

impl<A,C> Deref for AlphaData<A,C> {
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



// =========================
// === Rgb <-> LinearRgb ===
// =========================

fn into_linear<T:Component>(t:T) -> T {
    let one  : T = 1.0.into();
    let bend : T = 0.04045.into();
    let branch1  = (bend - t.clone()).signum().clamp(0.0.into(),1.0.into());
    let branch2  = one - branch1.clone();
    let val1     = t.clone() / 12.92.into();
    let val2     = ((t + 0.055.into()) / 1.055.into()).pow(2.4.into());
    branch1 * val1 + branch2 * val2
}

fn from_linear<T:Component>(t:T) -> T {
    let one  : T = 1.0.into();
    let bend : T = 0.0031308.into();
    let branch1  = (bend - t.clone()).signum().clamp(0.0.into(),1.0.into());
    let branch2  = one.clone() - branch1.clone();
    let val1     = t.clone() * 12.92.into();
    let val2     = t.pow(one / 2.4.into()) * 1.055.into() - 0.055.into();
    branch1 * val1 + branch2 * val2
}

impl<T:Component> From<Rgb<T>> for LinearRgb<T> {
    fn from(rgb:Rgb<T>) -> Self {
        from_components(rgb.map(|t| into_linear(t.clone())).to_components())
    }
}

impl<T:Component> From<LinearRgb<T>> for Rgb<T> {
    fn from(rgb:LinearRgb<T>) -> Self {
        from_components(rgb.map(|t| from_linear(t.clone())).to_components())
    }
}



// ===================
// === Rgb <-> Hsl ===
// ===================

impl From<Rgb> for Hsl {
    fn from(color:Rgb) -> Self {
        let min       = color.red.min(color.green).min(color.blue);
        let max       = color.red.max(color.green).max(color.blue);
        let luminance = (max + min) / 2.0;
        if(max == min){
            let hue        = 0.0;
            let saturation = 0.0;
            let data       = HslData {hue,saturation,luminance};
            Self {data}
        } else {
            let spread     = max - min;
            let saturation = if luminance > 0.5 { spread/(2.0-max-min) } else { spread/(max+min) };
            let red_dist   = if (color.green < color.blue) { 6.0 } else { 0.0 };
            let mut hue =
                if      color.red   == max { (color.green - color.blue)  / spread + red_dist }
                else if color.green == max { (color.blue  - color.red)   / spread + 2.0 }
                else                       { (color.red   - color.green) / spread + 4.0 };
            hue /= 6.0;
            let data = HslData {hue,saturation,luminance};
            Self {data}
        }
    }
}



// ===================
// === Rgb <-> Xyz ===
// ===================

/// Assumed D65 white point.
/// http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
impl From<LinearRgb> for Xyz {
    fn from(rgb:LinearRgb) -> Self {
        let r = rgb.red;
        let g = rgb.green;
        let b = rgb.blue;
        let x =  0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
        let y =  0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
        let z =  0.0193339 * r + 0.1191920 * g + 0.9503041 * b;
        let data = XyzData {x,y,z};
        Self {data}
    }
}

impl From<Xyz> for LinearRgb {
    fn from(xyz:Xyz) -> Self {
        let x = xyz.x;
        let y = xyz.y;
        let z = xyz.z;
        let red   =   3.2404542 * x + -1.5371385 * y + -0.4985314 * z;
        let green =  -0.9692660 * x +  1.8760108 * y +  0.0415560 * z;
        let blue  =   0.0556434 * x + -0.2040259 * y +  1.0572252 * z;
        let data  = LinearRgbData {red,green,blue};
        Self {data}
    }
}



convert_via! { Rgb -> LinearRgb -> Xyz }
convert_via! { Xyz -> LinearRgb -> Rgb }



// ===================
// === Xyz <-> Lab ===
// ===================

impl From<Xyz> for Lab {
    fn from(xyz:Xyz) -> Self {
        fn convert(c:f32) -> f32 {
            let epsilon : f32 = 6.0/29.0;
            let epsilon = epsilon.powi(3);
            let kappa   = 841.0/108.0;
            let delta   = 4.0/29.0;
            if c > epsilon { c.cbrt() } else { (kappa * c) + delta }
        }

        let xyz = xyz / D65::get_xyz();

        let x = convert(xyz.x);
        let y = convert(xyz.y);
        let z = convert(xyz.z);

        let l = (y * 116.0) - 16.0;
        let a = (x - y) * 500.0;
        let b = (y - z) * 200.0;

        let data = LabData {l,a,b};
        Self {data}
    }
}

impl From<Lab> for Xyz {
    fn from(color:Lab) -> Self {
        let y = (color.l + 16.0) / 116.0;
        let x = y + (color.a / 500.0);
        let z = y - (color.b / 200.0);

        fn convert(c:f32) -> f32 {
            let epsilon = 6.0 / 29.0;
            let kappa   = 108.0 / 841.0;
            let delta   = 4.0 / 29.0;
            if c > epsilon { c.powi(3) } else { (c - delta) * kappa }
        }

        Xyz::from_components((convert(x), convert(y), convert(z))) * D65::get_xyz()
    }
}

pub fn test() {
    let rgb = Rgb::from_components((0.2,0.4,0.6));
    let hsl = Hsl::from(rgb);
    let xyz = Xyz::from(rgb);
    println!("{:?}",hsl);
    println!("{:?}",xyz);
}
