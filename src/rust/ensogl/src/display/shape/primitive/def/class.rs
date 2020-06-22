//! This module defines the class of all shapes.

use crate::prelude::*;

use super::unit::*;
use super::modifier::*;

use crate::display::shape::primitive::shader::canvas;
use crate::display::shape::primitive::shader::canvas::Canvas;
use crate::display::shape::primitive::def::var::Var;
use crate::data::color;



// =============
// === Shape ===
// =============

/// Type of any shape which we can display on the canvas.
pub trait Shape = 'static + canvas::Draw;

/// Generic 2d shape representation. You can convert any specific shape type to this type and use it
/// as a generic shape type.
#[derive(Debug,Clone,CloneRef)]
pub struct AnyShape {
    rc: Rc<dyn canvas::Draw>
}

impl AsOwned for AnyShape {
    type Owned = AnyShape;
}

impl AnyShape {
    /// Constructor.
    pub fn new<T:Shape>(t:T) -> Self {
        Self {rc : Rc::new(t)}
    }
}

impl canvas::Draw for AnyShape {
    fn draw(&self, canvas:&mut Canvas) -> canvas::Shape {
        self.rc.draw(canvas)
    }
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

    /// Unwraps the shape and provides the raw reference to its content.
    pub fn unwrap(&self) -> &T {
        self.deref()
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



// ================
// === ShapeOps ===
// ================

impl<T> ShapeOps for ShapeRef<T> {}
impl    ShapeOps for AnyShape {}

/// Methods implemented by every shape.
pub trait ShapeOps : Sized where for<'t> &'t Self : IntoOwned<Owned=Self> {
    /// Translate the shape by a given offset.
    fn translate<V:Into<Var<Vector2<Distance<Pixels>>>>>(&self, v:V) -> Translate<Self> {
        Translate(self,v)
    }

    /// Translate the shape along X-axis by a given offset.
    fn translate_x<X>(&self, x:X) -> Translate<Self>
        where (X,Var<Distance<Pixels>>) : Into<Var<Vector2<Distance<Pixels>>>> {
        self.translate((x,0.px()))
    }

    /// Translate the shape along Y-axis by a given offset.
    fn translate_y<Y>(&self, y:Y) -> Translate<Self>
        where (Var<Distance<Pixels>>,Y) : Into<Var<Vector2<Distance<Pixels>>>> {
        self.translate((0.px(),y))
    }

    /// Rotate the shape by a given angle.
    fn rotate<A:Into<Var<Angle<Radians>>>>(&self, angle:A) -> Rotation<Self> {
        Rotation(self,angle)
    }

    /// Scales the shape by a given value.
    fn scale<S:Into<Var<f32>>>(&self, value:S) -> Scale<Self> {
        Scale(self,value)
    }

    /// Unify the shape with another one.
    fn union<S:IntoOwned>(&self, that:S) -> Union<Self,Owned<S>> {
        Union(self,that)
    }

    /// Subtracts the argument from this shape.
    fn difference<S:IntoOwned>(&self, that:S) -> Difference<Self,Owned<S>> {
        Difference(self,that)
    }

    /// Computes the intersection of the shapes.
    fn intersection<S:IntoOwned>(&self, that:S) -> Intersection<Self,Owned<S>> {
        Intersection(self,that)
    }

    /// Fill the shape with the provided color.
    fn fill<Color:Into<Var<color::Rgba>>>(&self, color:Color) -> Fill<Self> {
        Fill(self,color)
    }

    /// Makes the borders of the shape crisp. Please note that it removes any form of antialiasing
    /// and can cause distortions especially with round surfaces.
    fn pixel_snap(&self) -> PixelSnap<Self> {
        PixelSnap(self)
    }

    /// Grows the shape by the given amount.
    fn grow<T:Into<Var<Distance<Pixels>>>>(&self, value:T) -> Grow<Self> {
        Grow(self, value.into())
    }

    /// Shrinks the shape by the given amount.
    fn shrink<T:Into<Var<Distance<Pixels>>>>(&self, value:T) -> Shrink<Self> {
        Shrink(self, value.into())
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
