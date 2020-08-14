//! This module defines `Color` and `Alpha`, generic data types used to define specific color
//! implementations.

use crate::prelude::*;

use enso_generics::*;

use super::component::*;
use super::component::HasComponents;
use nalgebra::Vector3;
use nalgebra::Vector4;



// =============
// === Color ===
// =============

/// A wrapper for every color definition. The wrapper makes it easy to create some generic traits
/// for colors. Also, if you want your trait to implement on just any color, no matter the color
/// space or if it contains transparency, you can implement it for this wrapper.
///
/// Please note that each color can be converted to a generic component representation (tuple of
/// fields). No matter the underlying representation, `Color` defines math operations on the
/// components. You can for example always add or multiply colors component-wise. Although it is
/// not always correct, for example blending colors in sRGB space is just wrong, you sometimes may
/// just want it, for example to match the behavior of color mixing in web browsers, which is
/// broken for many years already:
/// https://stackoverflow.com/questions/60179850/webgl-2-0-canvas-blending-with-html-in-linear-color-space
#[derive(Clone,Copy,Default,PartialEq)]
pub struct Color<D> {
    /// The underlying color representation. It is either `Alpha` or a color space instance.
    pub data : D
}

/// Smart constructor.
#[allow(non_snake_case)]
pub fn Color<D>(data:D) -> Color<D> {
    Color {data}
}


// === Deref ===

impl<D> Deref for Color<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}


// === Component Generics ===

impl<D:HasComponentsRepr> HasComponentsRepr for Color<D> {
    type ComponentsRepr = ComponentsReprOf<D>;
}

impl<D:ComponentMap> ComponentMap for Color<D> {
    fn map<F:Fn(f32)->f32>(&self, f:F) -> Self {
        Self {data:self.data.map(f)}
    }
}


// === Conversions ===

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

impl<D> Into<Vector3<f32>> for Color<D>
where Color<D> : HasComponents<ComponentsRepr=(f32,f32,f32)> {
    fn into(self) -> Vector3<f32> {
        Into::<Vector3<f32>>::into(self.into_components())
    }
}

impl<D> Into<Vector4<f32>> for Color<D>
where Color<D> : HasComponents<ComponentsRepr=(f32,f32,f32,f32)> {
    fn into(self) -> Vector4<f32> {
        Into::<Vector4<f32>>::into(self.into_components())
    }
}

impl<D> Into<Vector3<f32>> for &Color<D>
where Color<D> : Copy + HasComponents<ComponentsRepr=(f32,f32,f32)> {
    fn into(self) -> Vector3<f32> {
        Into::<Vector3<f32>>::into((*self).into_components())
    }
}

impl<D> Into<Vector4<f32>> for &Color<D>
where Color<D> : Copy + HasComponents<ComponentsRepr=(f32,f32,f32,f32)> {
    fn into(self) -> Vector4<f32> {
        Into::<Vector4<f32>>::into((*self).into_components())
    }
}

impl<D> From<Vector3<f32>> for Color<D>
    where D : HasComponents<ComponentsRepr=(f32,f32,f32)> {
    fn from(t:Vector3<f32>) -> Self {
        Self::from(t.into_components())
    }
}

impl<D> From<Vector4<f32>> for Color<D>
    where D : HasComponents<ComponentsRepr=(f32,f32,f32,f32)> {
    fn from(t:Vector4<f32>) -> Self {
        Self::from(t.into_components())
    }
}

impl<D> From<&Vector3<f32>> for Color<D>
    where D : Copy + HasComponents<ComponentsRepr=(f32,f32,f32)> {
    fn from(t:&Vector3<f32>) -> Self {
        Self::from((*t).into_components())
    }
}

impl<D> From<&Vector4<f32>> for Color<D>
    where D : Copy + HasComponents<ComponentsRepr=(f32,f32,f32,f32)> {
    fn from(t:&Vector4<f32>) -> Self {
        Self::from((*t).into_components())
    }
}


// === Operators ===

/// Defines operators per-component. See `Color` docs to learn more.
macro_rules! define_color_operators {
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

define_color_operators!{ Add::add, Sub::sub, Mul::mul, Div::div }



// =============
// === Alpha ===
// =============

/// An alpha component wrapper for colors. 0.0 is fully transparent and 1.0 is fully opaque.
#[derive(Clone,Copy,PartialEq)]
#[allow(missing_docs)]
#[allow(missing_debug_implementations)]
pub struct Alpha<C> {
    pub alpha  : f32,
    pub opaque : Color<C>,
}


// === Component Generics ===

impl<C> HasComponentsRepr for Alpha<C>
where C:HasComponentsRepr, ComponentsReprOf<C>:PushBack<f32> {
    type ComponentsRepr = <ComponentsReprOf<C> as PushBack<f32>>::Output;
}

impl<C> From<Alpha<C>> for ComponentsOf<Alpha<C>>
where C:HasComponents, ComponentsReprOf<C>:PushBack<f32> {
    fn from(t:Alpha<C>) -> Self {
        t.opaque.data.into().push_back(t.alpha)
    }
}

impl<C> From<ComponentsOf<Alpha<C>>> for Alpha<C>
where C:HasComponents, ComponentsReprOf<C>:PushBack<f32>,
      <ComponentsReprOf<C> as PushBack<f32>>::Output : PopBack<Last=f32,Init=ComponentsReprOf<C>> {
    fn from(components:ComponentsOf<Self>) -> Self {
        let (alpha,init) = components.pop_back();
        let opaque        = from_components(init);
        Self {alpha,opaque}
    }
}

impl<C:ComponentMap> ComponentMap for Alpha<C> {
    fn map<F:Fn(f32)->f32>(&self, f:F) -> Self {
        let alpha  = f(self.alpha);
        let opaque = self.opaque.map(f);
        Self {alpha,opaque}
    }
}

impl<C> Deref for Alpha<C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        &self.opaque
    }
}

impl<C> From<C> for Alpha<C> {
    fn from(data:C) -> Self {
        let alpha  = 1.0;
        let opaque = Color {data};
        Self {alpha,opaque}
    }
}

impl<C:Default> Default for Alpha<C> {
    fn default() -> Self {
        let alpha  = 1.0;
        let opaque = default();
        Self {alpha,opaque}
    }
}
