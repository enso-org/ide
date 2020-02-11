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
pub trait Shape: Clone {
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

impl<T> ShapeRef<T> where ShapeRef<T>:Shape {
    /// Translate the shape by a given offset.
    pub fn translate<X:ShaderData<f32>,Y:ShaderData<f32>>(&self, x:X, y:Y) -> Translate<Self> {
        Translate(self,x,y)
    }

    /// Rotate the shape by a given angle.
    pub fn rotate<A:ShaderData<f32>>(&self, angle:A) -> Rotation<Self> {
        Rotation(self,angle)
    }

    /// Unify the shape with another one.
    pub fn union<S:Shape>(&self, that:&S) -> Union<Self,S> {
        Union(self,that)
    }

    /// Subtracts the argument from this shape.
    pub fn difference<S:Shape>(&self, that:&S) -> Difference<Self,S> {
        Difference(self,that)
    }

    /// Computes the intersection of the shapes.
    pub fn intersection<S:Shape>(&self, that:&S) -> Intersection<Self,S> {
        Intersection(self,that)
    }

    /// Fill the shape with the provided color.
    pub fn fill<Color:Into<Glsl>>(&self, color:Color) -> Fill<Self> {
        Fill(self,color)
    }
}

impl<T,S:Shape> Add<&S> for &ShapeRef<T> where ShapeRef<T>:Shape {
    type Output = Union<ShapeRef<T>,S>;
    fn add(self, that:&S) -> Self::Output {
        self.union(that)
    }
}

impl<T,S:Shape> Sub<&S> for &ShapeRef<T> where ShapeRef<T>:Shape {
    type Output = Difference<ShapeRef<T>,S>;
    fn sub(self, that:&S) -> Self::Output {
        self.difference(that)
    }
}

impl<T,S:Shape> Mul<&S> for &ShapeRef<T> where ShapeRef<T>:Shape {
    type Output = Intersection<ShapeRef<T>,S>;
    fn mul(self, that:&S) -> Self::Output {
        self.intersection(that)
    }
}
