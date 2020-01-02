//! This module defines the class of all shapes. Read the `Shape` documentation to learn more.


use crate::prelude::*;

use crate::display::shape::primitive::def::*;
use crate::display::shape::primitive::shader::canvas::Canvas;
use crate::display::shape::primitive::shader::canvas::CanvasShape;
use crate::display::shape::primitive::shader::canvas::Drawable;



// =============
// === HasId ===
// =============

/// Each shape definition has to be assigned with an unique id in order for the painter to
/// implement results cache. For example, we can create a circle as `s1` and then move it right,
/// which will result in the `s2` object. We can merge them together creating `s3` object. The
/// painter needs to discover that `s3` was in fact created from two `s1` under the hood.
///
/// This trait should not be implemented manually. It is implemented by `ShapeRef`, which
/// wraps every shape definition.
pub trait HasId {
    /// The id of a shape.
    fn id(&self) -> usize;
}



// =============
// === Shape ===
// =============

/// Type of every shape. Under the hood, every shape is `ShapeRef<P>` where `P:PrimShape`,
/// however, it is much easier to express the dependencies on more general type bounds, so the
/// following type does not mention the specific implementation details.
pub trait Shape = Drawable + HasId + Clone;



// ================
// === ShapeRef ===
// ================

/// Wrapper for primitive shapes. It makes them both immutable as well as assigns each shape with
/// an unique id.
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

impl<T:Drawable> ShapeRef<T> {
    /// Translate the shape by a given offset.
    pub fn translate(&self,x:f32,y:f32) -> Translate<Self> {
        Translate(self,x,y)
    }

    /// Unify the shape with another one.
    pub fn union<S:Shape>(&self,that:&S) -> Union<Self,S> {
        Union(self,that)
    }
}

impl<T> HasId for ShapeRef<T> {
    fn id(&self) -> usize {
        Rc::downgrade(&self.rc).as_raw() as *const() as usize
    }
}

impl<T:Drawable> Drawable for ShapeRef<T> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        self.rc.draw(canvas)
    }
}

impl<T:Drawable,S:Shape> std::ops::Add<&S> for &ShapeRef<T> {
    type Output = Union<ShapeRef<T>,S>;
    fn add(self, that:&S) -> Self::Output {
        self.union(that)
    }
}
