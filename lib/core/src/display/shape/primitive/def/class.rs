//! This module defines the class of all shapes.

use crate::prelude::*;

use crate::display::shape::primitive::def::*;
use crate::display::shape::primitive::shader::canvas::Canvas;
use crate::display::shape::primitive::shader::canvas::CanvasShape;
use crate::display::shape::primitive::shader::data::ShaderData;
use crate::system::gpu::shader::glsl::Glsl;

use std::ops::Sub;
use std::ops::Mul;



// =============
// === Shape ===
// =============

pub trait AsOwned {
    type Owned;
}

impl<T> AsOwned for &T {
    type Owned = T;
}

pub type Owned<T> = <T as AsOwned>::Owned;

pub trait IntoOwned = AsOwned + Into<Owned<Self>>;



/// Type of every shape. Under the hood, every shape is `ShapeRef<P>`, however, we do not use
/// specific `ShapeRef<P>` field here, as it is much easier to express any bounds when using
/// more generic types.
pub trait Shape: Clone + for<'t> From<&'t Self> {
    /// Draw the element on the canvas.
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape;
}



// ================
// === ShapeRef ===
// ================

/// Immutable reference to a shape. It is also used to get unique id for each shape.
#[derive(Debug,Derivative,Shrinkwrap)]
#[derivative(Clone(bound=""))]
pub struct ShapeRef<T> {
    rc:Rc<T>
}

impl<T> From<&ShapeRef<T>> for ShapeRef<T> {
    fn from(t:&ShapeRef<T>) -> Self {
        t.clone()
    }
}

impl<T> ShapeRef<T> {
    /// Constructor.
    pub fn new(t:T) -> Self {
        Self {rc:Rc::new(t)}
    }
}

impl<T> ShapeRef<T> {
    /// Each shape definition has to be assigned with an unique id in order for the painter to
    /// implement results cache. For example, we can create three shapes `s1`, `s2`, and `s3`. We
    /// want to define `s4 = s1 - s2`, `s5 = s1 - s3`, and `s6 = s4 + s5`. We need to discover that
    /// we use `s1` twice under the hood in order to optimize the GLSL.
    pub fn id(&self) -> usize {
        Rc::downgrade(&self.rc).as_raw() as *const() as usize
    }
}

impl<T> ShapeRef<T> {
    /// Translate the shape by a given offset.
    pub fn translate<X:ShaderData<f32>,Y:ShaderData<f32>>(&self, x:X, y:Y) -> Translate<Self> {
        Translate(self,x,y)
    }

    /// Rotate the shape by a given angle.
    pub fn rotate<A:ShaderData<f32>>(&self, angle:A) -> Rotation<Self> {
        Rotation(self,angle)
    }

    /// Unify the shape with another one.
    pub fn union<S:IntoOwned>(&self, that:S) -> Union<Self,Owned<S>> {
        Union(self,that)
    }

    /// Subtracts the argument from this shape.
    pub fn difference<S:IntoOwned>(&self, that:S) -> Difference<Self,Owned<S>> {
        Difference(self,that)
    }

    /// Computes the intersection of the shapes.
    pub fn intersection<S:IntoOwned>(&self, that:S) -> Intersection<Self,Owned<S>> {
        Intersection(self,that)
    }

    /// Fill the shape with the provided color.
    pub fn fill<Color:Into<Glsl>>(&self, color:Color) -> Fill<Self> {
        Fill(self,color)
    }
}

macro_rules! define_shape_operator {
    ($($op_trait:ident :: $op:ident => $shape_trait:ident :: $shape:ident)*) => {$(
        impl<T,S:IntoOwned> $op_trait<S> for &ShapeRef<T> {
            type Output = $shape_trait<ShapeRef<T>,Owned<S>>;
            fn $op(self, that:S) -> Self::Output {
                self.$shape(that)
            }
        }

        impl<T,S:IntoOwned> $op_trait<S> for ShapeRef<T> {
            type Output = $shape_trait<ShapeRef<T>,Owned<S>>;
            fn $op(self, that:S) -> Self::Output {
                self.$shape(that)
            }
        }
    )*}
}

define_shape_operator! {
    Add :: add => Union        :: union
    Sub :: sub => Difference   :: difference
    Mul :: mul => Intersection :: intersection
}
