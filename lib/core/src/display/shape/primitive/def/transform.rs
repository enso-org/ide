//! This module contains definitions of all primitive shapes transformations, like translation, or
//! rotation.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::prelude::*;

use crate::display::shape::primitive::def::class::Owned;
use crate::display::shape::primitive::def::class::AsOwned;
use crate::display::shape::primitive::def::class::IntoOwned;
use crate::display::shape::primitive::def::class::Drawable;
use crate::display::shape::primitive::def::class::Shape;
use crate::display::shape::primitive::def::class::ShapeRef;
use crate::display::shape::primitive::shader::canvas::Canvas;
use crate::display::shape::primitive::shader::canvas::CanvasShape;
use crate::display::shape::primitive::shader::data::ShaderData;
use crate::system::gpu::shader::glsl::Glsl;
use crate::system::gpu::types::*;
use crate::display::shape::primitive::def::sdf::{Distance,Pixels};



// ========================================
// === Compound Shape Definition Macros ===
// ========================================

/// Defines compound canvas shapes.
macro_rules! define_compound_shapes {
    ( $($name:ident $shapes:tt $fields:tt)* ) => {
        /// Contains mutable shapes definitions.
        pub mod mutable {
            use super::*;
            $(_define_compound_shape_data! {$name $shapes $fields})*
        }

        /// Contains immutable shapes definitions.
        pub mod immutable {
            use super::*;
            $(_define_compound_shape! {$name $shapes $fields})*
        }
    }
}

macro_rules! _define_compound_shape_data {
    ($name:ident ($($shape_field:ident),*$(,)?) ($($field:ident : $field_type:ty),*$(,)?)) => {

        /// Shape type definition.
        #[allow(missing_docs)]
        #[derive(Debug)]
        pub struct $name<$($shape_field),*> {
            $(pub $shape_field : $shape_field),*,
            $(pub $field       : Glsl),*
        }
        impl<$($shape_field),*> $name<$($shape_field),*> {
            /// Constructor.
            pub fn new<$($field:ShaderData<$field_type>),*>
            ($($shape_field:$shape_field),*,$($field:$field),*) -> Self {
                $(let $field = $field.into();)*
                Self {$($shape_field),*,$($field),*}
            }
        }

        impl<$($shape_field),*> AsOwned for $name<$($shape_field),*> { type Owned = $name<$($shape_field),*>; }

    }
}

macro_rules! _define_compound_shape {
    ($name:ident ($($shape_field:ident),*$(,)?) ($($field:ident : $field_type:ty),*$(,)?)) => {
        /// Shape type definition.
        pub type $name<$($shape_field),*> =
            ShapeRef<mutable::$name<$($shape_field),*>>;

        /// Smart constructor.
        pub fn $name<$($shape_field:IntoOwned),*,$($field:ShaderData<$field_type>),*>
        ( $($shape_field:$shape_field),*,$($field:$field),*) -> $name<$(Owned<$shape_field>),*> {
            ShapeRef::new(mutable::$name::new($($shape_field.into()),*,$($field),*))
        }

        impl<$($shape_field),*> AsOwned for $name<$($shape_field),*> {
            type Owned = $name<$($shape_field),*>;
        }

        impl<$($shape_field:Drawable),*> From<$name<$($shape_field),*>> for Shape {
            fn from(t:$name<$($shape_field),*>) -> Self {
                Self::new(t)
            }
        }
    }
}



// =======================
// === Compound Shapes ===
// =======================

use immutable::*;

define_compound_shapes! {
    Translate(child)(v:Vector2<Distance<Pixels>>)
    Rotation(child)(angle:f32)
    Scale(child)(value:f32)
    Union(child1,child2)()
    Difference(child1,child2)()
    Intersection(child1,child2)()
    Fill(child)(color:dyn Any)
    PixelSnap(child)()
}


impl<Child:Drawable> Drawable for Translate<Child> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child.draw(canvas);
        canvas.translate(self.id(),s1,&self.v)
    }
}

impl<Child:Drawable> Drawable for Rotation<Child> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child.draw(canvas);
        canvas.rotation(self.id(),s1,&self.angle)
    }
}

impl<Child:Drawable> Drawable for Scale<Child> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child.draw(canvas);
        canvas.scale(self.id(),s1,&self.value)
    }
}

impl<Child1:Drawable,Child2:Drawable> Drawable for Union<Child1,Child2> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child1.draw(canvas);
        let s2 = self.child2.draw(canvas);
        canvas.union(self.id(),s1,s2)
    }
}

impl<Child1:Drawable,Child2:Drawable> Drawable for Difference<Child1,Child2> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child1.draw(canvas);
        let s2 = self.child2.draw(canvas);
        canvas.difference(self.id(),s1,s2)
    }
}

impl<Child1:Drawable,Child2:Drawable> Drawable for Intersection<Child1,Child2> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child1.draw(canvas);
        let s2 = self.child2.draw(canvas);
        canvas.intersection(self.id(),s1,s2)
    }
}

impl<Child:Drawable> Drawable for Fill<Child> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s = self.child.draw(canvas);
        canvas.fill(self.id(),s,&self.color)
    }
}

impl<Child:Drawable> Drawable for PixelSnap<Child> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s = self.child.draw(canvas);
        canvas.pixel_snap(self.id(),s)
    }
}
