//! This module defines the class of all shapes.


use crate::prelude::*;

use crate::display::shape::primitive::def::*;
use crate::display::shape::primitive::shader::canvas::Canvas;
use crate::display::shape::primitive::shader::canvas::CanvasShape;



// =============
// === Shape ===
// =============

/// Type of every shape. Under the hood, every shape is `ShapeRef<P>`, however, it is much easier
/// to express the dependencies on more general type bounds, so the following type does not mention
/// the specific implementation details.
pub trait Shape: Clone {
    /// Draw the element on the canvas.
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape;
}



// ================
// === ShapeRef ===
// ================

/// Immutable wrapper for primitive shapes with fast clone operation. It also assigns each shape
/// with an unique id.
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
    pub fn translate(&self,x:f32,y:f32) -> Translate<Self> {
        Translate(self,x,y)
    }

    /// Unify the shape with another one.
    pub fn union<S:Shape>(&self,that:&S) -> Union<Self,S> {
        Union(self,that)
    }
}

impl<T,S:Shape> std::ops::Add<&S> for &ShapeRef<T> where ShapeRef<T>:Shape {
    type Output = Union<ShapeRef<T>,S>;
    fn add(self, that:&S) -> Self::Output {
        self.union(that)
    }
}
