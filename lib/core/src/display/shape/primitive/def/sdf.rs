//! This module defines all primitive Signed Distance Field (SDF) shapes.
//! Learn more about SDFs: https://en.wikipedia.org/wiki/Signed_distance_function

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::prelude::*;

use inflector::Inflector;

use crate::display::shape::primitive::def::class::ShapeRef;
use crate::display::shape::primitive::shader::canvas::Canvas;
use crate::display::shape::primitive::shader::canvas::CanvasShape;
use crate::display::shape::primitive::shader::canvas::Drawable;
use crate::display::shape::primitive::shader::item::GlslItem;
use crate::display::symbol::geometry::primitive::mesh::buffer::item::Item;



// =================
// === PrimShape ===
// =================

/// Class of primitive shapes. Primitive shapes are described by a SDF field.
pub trait PrimShape {
    /// Converts the shape to SDF GLSL code.
    fn to_sdf_code(&self) -> String;
}



// ====================================
// === Prim Shape Definition Macros ===
// ====================================

/// Defines primitive shapes and appropriate shape wrappers.
///
/// Primitive shapes are defined in the `mutable` module, while the shape wrappers are placed in
/// the `immutable` module. The shape definition accepted by this macro is similar to both a struct
/// and a function definition. It's body should be defined as a valid GLSL code.
///
/// For the following input:
/// ```compile_fail
/// define_prim_shapes! {
///     Circle (radius:f32) {
///         return sdf(length(position)-radius, bbox_center(radius,radius));
///     }
/// ```
///
/// The following output will be generated:
/// ```compile_fail
/// pub mod mutable {
///     use super::*;
///
///     #[derive(Debug,Clone)]
///     pub struct Circle {
///         pub glsl_name : String,
///         pub radius    : String,
///     }
///
///     impl Circle {
///         pub fn new<radius:GlslItem<f32>>(radius:radius) -> Self {
///             let glsl_name = "circle".to_string();
///             let radius    = radius.to_glsl();
///             Self {glsl_name,radius}
///         }
///     }
///
///     impl PrimShape for Circle {
///            fn to_sdf_code(&self) -> String {
///                let body = "return sdf(length(position)-radius, bbox_center(radius,radius));";
///                let args = vec![
///                    "vec2 position".to_string(),
///                    format!("{} {}", <$f32 as Item>::gpu_type_name(), "radius")
///                    ].join(", ");
///                format!("sdf {} ({}) {{ {} }}",self.glsl_name,args,body)
///            }
///        }
///
///        impl Drawable for Circle {
///            fn paint(&self, painter:&mut Painter) -> CanvasShape {
///             let args = vec!["position", &self.radius].join(",");
///             let code = format!("{}({})",self.glsl_name,args);
///             canvas.define_shape(&code,None)
///            }
///        }
/// }
///
/// pub mod immutable {
///     use super::*;
///
///     pub type Circle = ShapeRef<mutable::Circle>;
///     pub fn Circle<radius:GlslItem<f32>>(radius:radius) -> Circle {
///         Shape::new(mutable::Circle::new(radius))
///     }
/// }
/// ```

macro_rules! define_prim_shapes {
    ( $($name:ident $args:tt $body:tt)* ) => {

        /// Contains mutable shapes definitions.
        pub mod mutable {
            use super::*;
            $(_define_mutable! {$name $args $body} )*
        }

        /// Contains immutable shapes definitions.
        pub mod immutable {
            use super::*;
            $(_define_shape! {$name $args $body} )*
        }
    };
}

/// See the docs of `define_prim_shapes`.
macro_rules! _define_shape {
    ( $name:ident ( $($field:ident : $field_type:ty),* $(,)? ) { $($code:tt)* } ) => {

        /// Smart shape type.
        pub type $name = ShapeRef<mutable::$name>;

        /// Smart shape constructor.
        pub fn $name <$($field:GlslItem<$field_type>),*> ( $($field : $field),* ) -> $name {
            ShapeRef::new(mutable::$name::new($($field),*))
        }
    }
}

/// See the docs of `define_prim_shapes`.
macro_rules! _define_mutable {
    ( $name:ident ( $($field:ident : $field_type:ty),* $(,)? ) { $($code:tt)* } ) => {

        /// The shape definition.
        #[allow(missing_docs)]
        #[derive(Debug,Clone)]
        pub struct $name {
            pub glsl_name : String,
            $(pub $field  : String),*
        }

        impl $name {
            /// Constructor.
            pub fn new <$($field:GlslItem<$field_type>),*> ( $($field : $field),* ) -> Self {
                let glsl_name = stringify!($name).to_snake_case();
                $(let $field = $field.to_glsl();)*
                Self {glsl_name,$($field),*}
            }
        }

        impl PrimShape for $name {
            fn to_sdf_code(&self) -> String {
                let body = stringify!($($code)*);
                let args = vec!["vec2 position".to_string(), $(
                    format!("{} {}", <$field_type as Item>::gpu_type_name(), stringify!($field))
                ),*].join(", ");
                format!("sdf {} ({}) {{ {} }}",self.glsl_name,args,body)
            }
        }

        impl Drawable for $name {
            fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
                let args = vec!["position", $(&self.$field),* ].join(",");
                let code = format!("{}({})",self.glsl_name,args);
                canvas.define_shape(&code)
            }
        }
    };
}



// ===================
// === Prim Shapes ===
// ===================

define_prim_shapes! {

    // === Infinite ===

    Plane () {
        return sdf(FLOAT_MIN,bbox_center(0.0,0.0));
    }

    HalfPlane () {
        return sdf(position.y, bbox_center(0.0,0.0))
    }

    Line (width:f32) {
        return sdf(abs(position.y)-width, bbox_center(0.0,width));
    }


    // === Ellipse ===

    Circle (radius:f32) {
        return sdf(length(position)-radius, bbox_center(radius,radius));
    }

    Ellipse (x_radius:f32, y_radius:f32) {
        float a2   = x_radius * x_radius;
        float b2   = y_radius * y_radius;
        float px2  = position.x * position.x;
        float py2  = position.y * position.y;
        float dist = (b2 * px2 + a2 * py2 - a2 * b2) / (a2 * b2);
        return sdf(dist, bbox_center(x_radius,y_radius));
    }


    // === Rectangle ===

    SharpRect (width:f32, height:f32) {
        vec2 size = vec2(width,height);
        return max_el(abs(position) - size);
    }

    Rect (width:f32, height:f32) {
        vec2  size = vec2(width,height);
        vec2  dir  = abs(position) - size;
        float dist = max_el(min(dir,0.0)) + length(max(dir,0.0));
        return sdf(dist,bbox_center(width,height));
    }

    RoundedRectByCorner
    (width:f32, height:f32, top_left:f32, top_right:f32, bottom_left:f32, bottom_right:f32) {
        vec2 size = vec2(width,height);
        size /= 2.0;

        float tl = top_left;
        float tr = top_right;
        float bl = bottom_left;
        float br = bottom_right;

        bool is_top_left     = position.x <  - size.x + tl && position.y >   size.y - tl;
        bool is_top_right    = position.x >    size.x - tr && position.y >   size.y - tr;
        bool is_bottom_left  = position.x <  - size.x + bl && position.y < - size.y + bl;
        bool is_bottom_right = position.x >    size.x - br && position.y < - size.y + br;

        if      is_top_left     {return length(position - vec2(- size.x + tl,   size.y - tl)) - tl;}
        else if is_top_right    {return length(position - vec2(  size.x - tr,   size.y - tr)) - tr;}
        else if is_bottom_left  {return length(position - vec2(- size.x + bl, - size.y + bl)) - bl;}
        else if is_bottom_right {return length(position - vec2(  size.x - br, - size.y + br)) - br;}
        else {
            vec2 dir = abs(position) - size;
            return min(max(dir.x,dir.y),0.0) + length(max(dir,0.0));
        }
    }


    // === Triangle ===

    Triangle (width:f32, height:f32) {
        vec2  norm = normalize(vec2(height,width/2.0));
        float dist = max(abs(position).x*norm.x + position.y*norm.y - height*norm.y, -position.y);
        return sdf(dist,bbox_center(width,height/2.0));
    }
}
