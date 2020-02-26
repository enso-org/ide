//! This module defines all primitive Signed Distance Field (SDF) shapes.
//! Learn more about SDFs: https://en.wikipedia.org/wiki/Signed_distance_function

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::prelude::*;

use inflector::Inflector;

use crate::display::shape::primitive::def::class::ShapeRef;
use crate::display::shape::primitive::shader::canvas;
use crate::display::shape::primitive::shader::canvas::Canvas;
use crate::display::shape::primitive::shader::canvas::CanvasShape;
use crate::display::shape::Var;
use crate::system::gpu::shader::glsl::Glsl;

use crate::system::gpu::shader::glsl::traits::*;
use crate::system::gpu::types::*;
use crate::math::topology::unit::*;



// ================
// === SdfShape ===
// ================

/// Class of primitive SDF shapes.
pub trait SdfShape {
    /// Gets the SDF definition for the given shape.
    fn glsl_definition() -> String;
}



// ====================================
// === Prim Shape Definition Macros ===
// ====================================

/// Defines SDF shapes and appropriate shape wrappers.
///
/// SDF shapes are defined in the `mutable` module, while the shape wrappers are placed in the
/// `immutable` module. The shape definition accepted by this macro is similar to both a struct
/// and a function definition.
///
/// The body of the shape definition should be a valid GLSL function body code. The function is
/// provided with two parameters:
///   - The current position point as `vec2 position`.
///   - All input parameters bound to this shader from the material definition as `Env env`.
///
/// The result of this shader should be a new `BoundSdf` instance. For more information about
/// the types and available helper functions in GLSL, please refer to the GLSL definitions in
/// `src/display/shape/primitive/def/glsl/*.glsl` files.
///
/// This macro will also generate a `all_shapes_glsl_definitions` function which returns a GLSL code
/// containing all shapes definitions in one place.

macro_rules! define_sdf_shapes {
    ( $($name:ident $args:tt $body:tt)* ) => {

        /// Contains mutable shapes definitions.
        pub mod mutable {
            use super::*;
            $(_define_sdf_shape_mutable_part! {$name $args $body} )*
        }

        $(_define_sdf_shape_immutable_part! {$name $args $body} )*

        /// GLSL definition of all shapes.
        pub fn all_shapes_glsl_definitions() -> String {
            vec![$($name::glsl_definition()),*].join("\n\n")
        }
    };
}

/// See the docs of `define_sdf_shapes`.
macro_rules! _define_sdf_shape_immutable_part {
    ( $name:ident ( $($field:ident : $field_type:ty),* $(,)? ) $body:tt ) => {

        /// Smart shape type.
        pub type $name = ShapeRef<mutable::$name>;

        /// Smart shape constructor.
        pub fn $name <$($field:Into<Var<$field_type>>),*> ( $($field : $field),* ) -> $name {
            ShapeRef::new(mutable::$name::new($($field),*))
        }

        impl canvas::Draw for $name {
            fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
                let args = vec!["position".to_string(), $(self.$field.glsl().into()),* ].join(",");
                let code = format!("{}({})",self.glsl_name,args);
                canvas.define_shape(self.id(),&code)
            }
        }

        impl SdfShape for $name {
            fn glsl_definition() -> String {
                let name = stringify!($name).to_snake_case();
                let body = stringify!($body);
                let args = vec!["vec2 position".to_string(), $(
                    format!("{} {}", <$field_type>::glsl_prim_type(), stringify!($field))
                ),*].join(", ");
                iformat!("BoundSdf {name} ({args}) {body}")
            }
        }

        impl AsOwned for $name { type Owned = $name; }

        impl $name {$(
            /// Field accessor.
            pub fn $field(&self) -> &Var<$field_type> {
                &self.unwrap().$field
            }
        )*}
    }
}

/// See the docs of `define_sdf_shapes`.
macro_rules! _define_sdf_shape_mutable_part {
    ( $name:ident ( $($field:ident : $field_type:ty),* $(,)? ) { $($code:tt)* } ) => {

        /// The shape definition.
        #[allow(missing_docs)]
        #[derive(Debug,Clone)]
        pub struct $name {
            pub glsl_name : Glsl,
            $(pub $field  : Var<$field_type>),*
        }

        impl $name {
            /// Constructor.
            #[allow(clippy::new_without_default)]
            pub fn new <$($field:Into<Var<$field_type>>),*> ( $($field : $field),* ) -> Self {
                let glsl_name = stringify!($name).to_snake_case().into();
                $(let $field = $field.into();)*
                Self {glsl_name,$($field),*}
            }
        }
    };
}









// ===================
// === Prim Shapes ===
// ===================

define_sdf_shapes! {

    // === Infinite ===

    Plane () {
        return bound_sdf(FLOAT_MIN,bounding_box(0.0,0.0));
    }

    HalfPlane () {
        return bound_sdf(position.y, bounding_box(0.0,0.0));
    }

    PlaneAngle (angle:AngleIn<Radians>) {
        float v_angle  = value(angle);
        float distance = abs(position).x*cos(v_angle/2.0) + -position.y*sin(v_angle/2.0) + 0.5;
        return bound_sdf(distance,bounding_box(0.0,0.0));
    }

    Line (width:f32) {
        return bound_sdf(abs(position.y)-width, bounding_box(0.0,width));
    }


    // === Ellipse ===

    Circle (radius:DistanceIn<Pixels>) {
        return bound_sdf(length(position)-radius, bounding_box(radius,radius));
    }

    Ellipse (x_radius:f32, y_radius:f32) {
        float a2   = x_radius * x_radius;
        float b2   = y_radius * y_radius;
        float px2  = position.x * position.x;
        float py2  = position.y * position.y;
        float dist = (b2 * px2 + a2 * py2 - a2 * b2) / (a2 * b2);
        return bound_sdf(dist, bounding_box(x_radius,y_radius));
    }


    // === Rectangle ===

    Rect (size:Vector2<DistanceIn<Pixels>>) {
        vec2  dir  = abs(position) - size/2.0;
        float dist = max(min(dir,0.0)) + length(max(dir,0.0));
        return bound_sdf(dist,bounding_box(size));
    }

    RoundedRectByCorner
    (size:Vector2<DistanceIn<Pixels>>, top_left:DistanceIn<Pixels>, top_right:DistanceIn<Pixels>, bottom_left:DistanceIn<Pixels>, bottom_right:DistanceIn<Pixels>) {
        size /= 2.0;

        float tl = top_left;
        float tr = top_right;
        float bl = bottom_left;
        float br = bottom_right;

        bool is_top_left     = position.x <  -size.x + tl && position.y >  size.y - tl;
        bool is_top_right    = position.x >   size.x - tr && position.y >  size.y - tr;
        bool is_bottom_left  = position.x <  -size.x + bl && position.y < -size.y + bl;
        bool is_bottom_right = position.x >   size.x - br && position.y < -size.y + br;

        float dist;
        if      (is_top_left)     {dist = length(position - vec2(-size.x + tl,  size.y - tl)) - tl;}
        else if (is_top_right)    {dist = length(position - vec2( size.x - tr,  size.y - tr)) - tr;}
        else if (is_bottom_left)  {dist = length(position - vec2(-size.x + bl, -size.y + bl)) - bl;}
        else if (is_bottom_right) {dist = length(position - vec2( size.x - br, -size.y + br)) - br;}
        else {
            vec2 dir = abs(position) - size;
            dist = min(max(dir.x,dir.y),0.0) + length(max(dir,0.0));
        }
        return bound_sdf(dist,bounding_box(size));
    }


    // === Triangle ===

    Triangle (width:f32, height:f32) {
        vec2  norm = normalize(vec2(height,width/2.0));
        float dist = max(abs(position).x*norm.x + position.y*norm.y - height*norm.y, -position.y);
        return bound_sdf(dist,bounding_box(width,height/2.0));
    }
}


impl Plane {
    /// Cuts angle from the plane.
    pub fn angle<T:Into<Var<AngleIn<Radians>>>>(&self, t:T) -> PlaneAngle {
        PlaneAngle(t)
    }
}

impl Rect {
    /// Sets the radius of all the corners.
    pub fn corners_radius<T>(&self, radius:T) -> RoundedRectByCorner
    where T:Into<Var<DistanceIn<Pixels>>> {
        let radius       = radius.into();
        let top_left     = radius.clone();
        let top_right    = radius.clone();
        let bottom_left  = radius.clone();
        let bottom_right = radius;
        RoundedRectByCorner(self.size(),top_left,top_right,bottom_left,bottom_right)
    }

    /// Sets the radiuses of each of the corners.
    pub fn corners_radiuses<T1,T2,T3,T4>
    (&self, top_left:T1, top_right:T2, bottom_left:T3, bottom_right:T4) -> RoundedRectByCorner
    where T1 : Into<Var<DistanceIn<Pixels>>> ,
          T2 : Into<Var<DistanceIn<Pixels>>> ,
          T3 : Into<Var<DistanceIn<Pixels>>> ,
          T4 : Into<Var<DistanceIn<Pixels>>> {
        RoundedRectByCorner(self.size(),top_left,top_right,bottom_left,bottom_right)
    }

    /// Sets the radiuses of the left corners.
    pub fn left_corners_radius<T>(&self, radius:T) -> RoundedRectByCorner
    where T:Into<Var<DistanceIn<Pixels>>> {
        let radius       = radius.into();
        let top_left     = radius.clone();
        let bottom_left  = radius;
        let top_right    = 0.px();
        let bottom_right = 0.px();
        RoundedRectByCorner(self.size(),top_left,top_right,bottom_left,bottom_right)
    }

    /// Sets the radiuses of the right corners.
    pub fn right_corners_radius<T>(&self, radius:T) -> RoundedRectByCorner
        where T:Into<Var<DistanceIn<Pixels>>> {
        let radius       = radius.into();
        let top_left     = 0.px();
        let bottom_left  = 0.px();
        let top_right    = radius.clone();
        let bottom_right = radius;
        RoundedRectByCorner(self.size(),top_left,top_right,bottom_left,bottom_right)
    }
}
