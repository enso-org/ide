//! Definition of style sheet values.

use crate::prelude::*;

use crate::data::color;
use palette::Hue;
use palette::Saturate;
use palette::Shade;



// ============
// === Data ===
// ============

/// Type of values in the style sheet.
#[derive(Debug,Clone,PartialEq)]
#[allow(missing_docs)]
pub enum Data {
    Invalid(String),
    Number(f32),
    Color(color::Lcha),
}


// === Constructors ===

/// Smart constructor for `Data`.
pub fn data<T:Into<Data>>(t:T) -> Data {
    t.into()
}

impl From<f32> for Data {
    fn from(t:f32) -> Data {
        Data::Number(t)
    }
}

impl<C,T> From<palette::Alpha<C,T>> for Data
where palette::Alpha<C,T> : Into<color::Lcha> {
    fn from(t:palette::Alpha<C,T>) -> Data {
        Data::Color(t.into())
    }
}



// === Impls ===

impl Display for Data {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(s) => write!(f,"{}",s),
            Self::Number(t)  => write!(f,"Number({})",t),
            Self::Color(t)   => write!(f,"Color({:?})",t),
        }
    }
}


macro_rules! define_color_transform {
    ($($name:ident),*) => {$(
        impl Data {
            /// Transform the color: $name.
            pub fn $name(&self, amount:Data) -> Self {
                match (self,amount) {
                    (Data::Invalid(s) , _)                => Data::Invalid(s.clone()),
                    (_                , Data::Invalid(s)) => Data::Invalid(s.clone()),
                    (Data::Color(t)   , Data::Number(f))  => Data::Color(t.$name(f)),
                    (this             , t)           => Data::Invalid
                        (format!(concat!("Cannot apply",stringify!($name),"({}) to {}."),t,this))
                }
            }
        }
    )*};
}

define_color_transform!(lighten,darken,saturate,desaturate,with_hue,shift_hue);


// === Color Getters ===

macro_rules! define_color_getter {
    ($($space:ident :: $name:ident),*) => {$(
        impl Data {
            /// Component getter.
            pub fn $name(&self) -> Data {
                match self {
                    Data::Invalid(s) => Data::Invalid(s.clone()),
                    Data::Color(t)   => Data::Number(palette::$space::from(*t).$name),
                    this             => Data::Invalid (format!
                        (concat!("Cannot access ",stringify!($name)," property of {}."),this))
                }
            }
        }
    )*};
}

define_color_getter!(Lcha::alpha);
define_color_getter!(LinSrgba::red,LinSrgba::green,LinSrgba::blue);


// === Operators ===

macro_rules! define_binary_number_operator {
    ($($toks:tt)*) => {
        _define_binary_number_operator! { [&Data] [&Data] $($toks)* }
        _define_binary_number_operator! { [ Data] [&Data] $($toks)* }
        _define_binary_number_operator! { [&Data] [ Data] $($toks)* }
        _define_binary_number_operator! { [ Data] [ Data] $($toks)* }
    };
}

macro_rules! _define_binary_number_operator {
    ([$($t1:tt)*] [$($t2:tt)*] $name:ident :: $fn:ident, $($err:tt)*) => {
        impl $name<$($t2)*> for $($t1)* {
            type Output = Data;
            #[allow(clippy::redundant_closure_call)]
            fn $fn(self, rhs:$($t2)*) -> Self::Output {
                match(self,rhs) {
                    (Data::Invalid(t),_) => Data::Invalid(t.clone()),
                    (_,Data::Invalid(t)) => Data::Invalid(t.clone()),
                    (Data::Number(lhs),Data::Number(rhs)) => Data::Number(lhs.$fn(rhs)),
                    (lhs,rhs) => Data::Invalid(($($err)*)(lhs,rhs))
                }
            }
        }
    };
}

define_binary_number_operator!(Mul::mul,|lhs,rhs| format!("Cannot multiply {} by {}.",lhs,rhs));
define_binary_number_operator!(Div::div,|lhs,rhs| format!("Cannot divide {} by {}.",lhs,rhs));
define_binary_number_operator!(Add::add,|lhs,rhs| format!("Cannot add {} to {}.",lhs,rhs));
define_binary_number_operator!(Sub::sub,|lhs,rhs| format!("Cannot subtract {} from {}.",rhs,lhs));



// =================
// === DataMatch ===
// =================

/// Smart `Data` deconstructors.
#[allow(missing_docs)]
pub trait DataMatch {
    fn invalid (&self) -> Option<&String>;
    fn number  (&self) -> Option<f32>;
    fn color   (&self) -> Option<color::Lcha>;
}

impl DataMatch for Data {
    fn invalid (&self) -> Option<&String>     {match self { Self::Invalid (t)=>Some(t)  , _=>None }}
    fn number  (&self) -> Option<f32>         {match self { Self::Number  (t)=>Some(*t) , _=>None }}
    fn color   (&self) -> Option<color::Lcha> {match self { Self::Color   (t)=>Some(*t) , _=>None }}
}

impl DataMatch for Option<Data> {
    fn invalid (&self) -> Option<&String>     {self.as_ref().and_then(|t| t.invalid())}
    fn number  (&self) -> Option<f32>         {self.as_ref().and_then(|t| t.number())}
    fn color   (&self) -> Option<color::Lcha> {self.as_ref().and_then(|t| t.color())}
}
