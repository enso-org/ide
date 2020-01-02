//! Shader primitives used to render vector shapes on GPU.

use crate::prelude::*;

use std::include_str;
use inflector::Inflector;
use crate::display::symbol::geometry::primitive::mesh::buffer::item::Item;
use nalgebra::Vector2;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;


const GLSL_DEFS:&str = include_str!("shader/defs.glsl");


// ================
// === GlslItem ===
// ================

/// Trait describing all types which can be converted to GLSL expressions.
///
/// `GlslItem<T>` is implemented for both `T` as well as for all kind of string inputs. This allows
/// for dirty injection of GLSL code easily. For example, when moving a shape, you can write
/// `s1.translate("a","b")`, where `a` and `b` refer to variables defined in the GLSL shader. Such
/// operation is not checked during compilation, so be careful when using it, please.

pub trait GlslItem<T> {
    /// Checks if the value is zero.
    fn is_zero (&self) -> bool;

    /// Converts the value to GLSL code.
    fn to_glsl (&self) -> String;
}


// === Instances ===

impl<T> GlslItem<T> for String {
    fn is_zero (&self) -> bool   { self == "0" || self == "0.0" }
    fn to_glsl (&self) -> String { self.into() }
}

impl<T> GlslItem<T> for &String {
    fn is_zero (&self) -> bool   { *self == "0" || *self == "0.0" }
    fn to_glsl (&self) -> String { (*self).into() }
}

impl<T> GlslItem<T> for str {
    fn is_zero (&self) -> bool   { self == "0" || self == "0.0" }
    fn to_glsl (&self) -> String { self.into() }
}

impl<T> GlslItem<T> for &str {
    fn is_zero (&self) -> bool   { *self == "0" || *self == "0.0" }
    fn to_glsl (&self) -> String { (*self).into() }
}

impl GlslItem<f32> for f32 {
    fn is_zero (&self) -> bool   { *self == 0.0 }
    fn to_glsl (&self) -> String {
        let is_int = self.fract() == 0.0;
        if is_int { iformat!("{self}.0") }
        else      { iformat!("{self}")   }
    }
}



// ===================
// === CanvasShape ===
// ===================

/// Reference to a shape defined on `Canvas`.
#[derive(Clone,Debug)]
pub struct CanvasShape {
    shape_num : usize,
    ids       : Vec<usize>,
    name      : String,
}

impl CanvasShape {
    /// Constructor.
    pub fn new(shape_num:usize) -> Self {
        let ids  = default();
        let name = format!("shape_{}",shape_num.to_string());
        Self {shape_num,ids,name}
    }

    /// Adds new id enclosed in this shape.
    pub fn add_id(&mut self, id:usize) {
        self.ids.push(id);
    }

    /// Add multiple ids enclosed in this shape.
    pub fn add_ids(&mut self, ids:&Vec<usize>) {
        self.ids.extend(ids)
    }

    /// Getter of the shape as GLSL expression.
    pub fn getter(&self) -> String {
        iformat!("{self.name}(global,position)")
    }
}



// ==============
// === Canvas ===
// ==============

// === Definition ===

/// Canvas for drawing vector graphics.
///
/// The API is stateful, similar to the API of HTML5 canvas element.
/// It uses GLSL and signed distance fields under the hood.

#[derive(Debug,Default)]
pub struct Canvas {
    next_shape_num         : usize,
    next_id                : usize,
    functions              : Vec<String>,
    current_function_lines : Vec<String>,
}


// === ID Management ===

impl Canvas {
    /// Generates a new unique shape's ID.
    fn get_new_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Generate a new unique shape number.
    pub fn get_new_shape_num(&mut self) -> usize {
        let out = self.next_shape_num;
        self.next_shape_num += 1;
        out
    }
}


// === GLSL Modification ===

impl Canvas {
    /// Adds new code line to the GLSL code.
    fn add_current_function_code_line<S:Str>(&mut self, line:S) {
        self.current_function_lines.push(format!("    {}",line.as_ref()));
    }

    /// Defines a new variable in the GLSL code.
    fn define<E:Str>(&mut self, ty:&str, name:&str, expr:E) {
        let max_type_length = 6;
        let max_name_length = 13;
        let ty              = format!("{:1$}" , ty   , max_type_length);
        let name            = format!("{:1$}" , name , max_name_length);
        self.add_current_function_code_line(iformat!("{ty} {name} = {expr.as_ref()};"));
    }

    /// Submits the `current_function_lines` as a new shape construction function in the GLSL code.
    fn submit_shape_constructor(&mut self, name:&str) {
        let body = self.current_function_lines.join("\n");
        let func = iformat!("shape {name} (globals global, vec2 position) {{\n{body}\n}}");
        self.current_function_lines = default();
        self.functions.push(func);
    }

    /// Get the final GLSL code.
    fn to_glsl(&self) -> String {
        if !self.current_function_lines.is_empty() {
            panic!("Internal error. Not all canvas GLSL code lines were converted to functions.");
        }
        self.functions.join("\n\n")
    }
}


// === Shape Definition ===

impl Canvas {
    /// Creates a new `CanvasShape` object. The shape is not assigned with any id and is not
    /// represented in the GLSL code yet.
    fn new_canvas_shape(&mut self) -> CanvasShape {
        let num = self.get_new_shape_num();
        CanvasShape::new(num)
    }

    /// Defines a new shape with a new id and associated parameters, like color.
    fn define_shape(&mut self, sdf:&str, cd:Option<&str>) -> CanvasShape {
        let color     = "rgb2lch(vec3(1.0,0.0,0.0)";
        let mut shape = self.new_canvas_shape();
        let id        = self.get_new_id();
        self.define("color" , "shape_color" , iformat!("{color}"));
        self.define("sdf"   , "shape_sdf"   , iformat!("{sdf}"));
        self.define("id"    , "shape_id"    , iformat!("new_id_layer(shape_sdf,{id})"));
        self.add_current_function_code_line("return shape(shape_id,shape_color,shape_sdf);");
        self.submit_shape_constructor(&shape.name);
        shape.add_id(id);
        shape
    }

    /// Define a new shape from the provided GLSL expression.
    pub fn new_shape_from_expr(&mut self, expr:&str) -> CanvasShape {
        let shape = self.new_canvas_shape();
        self.add_current_function_code_line(expr);
        self.submit_shape_constructor(&shape.name);
        shape
    }
}


// === Shape Modification ===

impl Canvas {
    /// Create a union shape from the provided shape components.
    pub fn union(&mut self, s1:CanvasShape, s2:CanvasShape) -> CanvasShape {
        let expr      = iformat!("return union({s1.getter()},{s2.getter()});");
        let mut shape = self.new_shape_from_expr(&expr);
        shape.add_ids(&s1.ids);
        shape.add_ids(&s2.ids);
        shape
    }

    /// Translate the current canvas origin.
    pub fn translate<X:GlslItem<f32>,Y:GlslItem<f32>>(&mut self, x:X, y:Y) {
        let expr = iformat!("sdf_translate(position, vec2({x.to_glsl()},{y.to_glsl()}))");
        self.define("","position",expr);
    }
}


// === Main API ===

impl Canvas {
    /// Returns the final GLSL code.
    pub fn run<S:Shape>(shape:&S) -> String {
        let mut canvas = Canvas::default();
        let shape_ref  = shape.draw(&mut canvas);
        canvas.add_current_function_code_line(iformat!("return {shape_ref.getter()};"));
        canvas.submit_shape_constructor("run");
        canvas.to_glsl()
    }
}



// =================
// === PrimShape ===
// =================

/// Class of primitive shapes. Primitive shapes are described by a SDF field.
pub trait PrimShape {
    fn to_sdf_code(&self) -> String;
}



// ====================================
// === Prim Shape Definition Macros ===
// ====================================

/// Defines primitive shapes and appropriate shape wrappers.
///
/// Primitive shapes are defined in the `prim_shape_data` module, while the shape wrappers are placed in
/// the `shapes` module. The shape definition accepted by this macro is similar to both a struct and
/// a function definition. It's body should be defined as a valid GLSL code.
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
/// pub mod prim_shape_data {
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
/// pub mod prim_shape {
///     use super::*;
///
///     pub type Circle = ImmutableShape<prim_shape_data::Circle>;
///     pub fn Circle<radius:GlslItem<f32>>(radius:radius) -> Circle {
///         Shape::new(prim_shape_data::Circle::new(radius))
///     }
/// }
/// ```

macro_rules! define_prim_shapes {
    ( $($name:ident $args:tt $body:tt)* ) => {
        pub mod prim_shape_data {
            use super::*;
            $(_define_prim_shape_data! {$name $args $body} )*
        }

        pub mod prim_shape {
            use super::*;
            $(_define_shape! {$name $args $body} )*
        }
    };
}

/// See the docs of `define_prim_shapes`.
macro_rules! _define_shape {
    ( $name:ident ( $($field:ident : $field_type:ty),* $(,)? ) { $($code:tt)* } ) => {

        /// Smart shape type.
        pub type $name = ImmutableShape<prim_shape_data::$name>;

        /// Smart shape constructor.
        pub fn $name <$($field:GlslItem<$field_type>),*> ( $($field : $field),* ) -> $name {
            ImmutableShape::new(prim_shape_data::$name::new($($field),*))
        }
    }
}

/// See the docs of `define_prim_shapes`.
macro_rules! _define_prim_shape_data {
    ( $name:ident ( $($field:ident : $field_type:ty),* $(,)? ) { $($code:tt)* } ) => {

        /// The shape definition.
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
                canvas.define_shape(&code,None)
            }
        }
    };
}



// ========================================
// === Compound Shape Definition Macros ===
// ========================================

/// Defines compound canvas shapes.
///
/// For the following input:
/// ```compile_fail
/// define_compound_shapes! {
///    Translate(child)(x:f32,y:f32)
/// }
/// ```
///
/// The macro generates:
/// ```compile_fail
/// pub mod compound_shape_data {
///     use super::*;
///
///     pub struct Translate<child> {
///         pub child : child,
///         pub x     : String,
///         pub y     : String,
///     }
///
///     impl<child:Shape> Translate<child> {
///         pub fn new<x:GlslItem<f32>,y:GlslItem<f32>>(child:&child,x:x,y:y) -> Self {
///             let child = child.clone();
///             let x     = x.to_glsl();
///             let y     = y.to_glsl();
///             Self {child,x,y}
///         }
///     }
/// }
///
/// pub mod compound_shape {
///     use super::*;
///
///     pub type Translate<child> = ImmutableShape<compound_shape_data::Translate<child>>;
///     pub fn Translate<child:Shape,x:GlslItem<f32>,y:GlslItem<f32>>
///     (child:&child,x:x,y:y) -> Translate<child> {
///         ImmutableShape::new(compound_shape_data::Translate::new(child,x,y))
///     }
/// }
/// ```

macro_rules! define_compound_shapes {
    ( $($name:ident $shapes:tt $fields:tt)* ) => {
        pub mod compound_shape_data {
            use super::*;
            $(_define_compound_shape_data! {$name $shapes $fields})*
        }
        pub mod compound_shape {
            use super::*;
            $(_define_compound_shape! {$name $shapes $fields})*
        }
    }
}

macro_rules! _define_compound_shape_data {
    ($name:ident ($($shape_field:ident),*$(,)?) ($($field:ident : $field_type:ty),*$(,)?)) => {
        pub struct $name<$($shape_field),*> {
            $(pub $shape_field : $shape_field),*,
            $(pub $field       : String      ),*
        }

        impl<$($shape_field:Shape),*> $name<$($shape_field),*> {
            pub fn new<$($field:GlslItem<$field_type>),*>
            ($($shape_field:&$shape_field),*,$($field:$field),*) -> Self {
                $(let $shape_field = $shape_field.clone();)*
                $(let $field       = $field.to_glsl();)*
                Self {$($shape_field),*,$($field),*}
            }
        }
    }
}

macro_rules! _define_compound_shape {
    ($name:ident ($($shape_field:ident),*$(,)?) ($($field:ident : $field_type:ty),*$(,)?)) => {
        pub type $name<$($shape_field),*> =
            ImmutableShape<compound_shape_data::$name<$($shape_field),*>>;
        pub fn $name<$($shape_field:Shape),*,$($field:GlslItem<$field_type>),*>
        ( $($shape_field:&$shape_field),*,$($field:$field),*) -> $name<$($shape_field),*> {
            ImmutableShape::new(compound_shape_data::$name::new($($shape_field),*,$($field),*))
        }
    }
}



// =============
// === HasId ===
// =============

/// Each shape definition has to be assigned with an unique id in order for the painter to
/// implement results cache. For example, we can create a circle as `s1` and then move it right,
/// which will result in the `s2` object. We can merge them together creating `s3` object. The
/// painter needs to discover that `s3` was in fact created from two `s1` under the hood.
///
/// This trait should not be implemented manually. It is implemented by `ImmutableShape`, which
/// wraps every shape definition.
pub trait HasId {
    fn id(&self) -> usize;
}



// ================
// === Drawable ===
// ================

/// Describes every shape which can be painted on the canvas.
trait Drawable {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape;
}



// ==============================
// === Shape & ImmutableShape ===
// ==============================

/// Type of every shape. Under the hood, every shape is `ImmutableShape<P>` where `P:PrimShape`,
/// however, it is much easier to express the dependencies on more general type bounds, so the
/// following type does not mention the specific implementation details.
pub trait Shape = Drawable + HasId + Clone;


// === ImmutableShape ===

/// Wrapper for primitive shapes. It makes them both immutable as well as assigns each shape with
/// an unique id.
#[derive(Debug,Derivative,Shrinkwrap)]
#[derivative(Clone(bound=""))]
pub struct ImmutableShape<T> {
    rc:Rc<T>
}

impl<T> ImmutableShape<T> {
    pub fn new(t:T) -> Self {
        Self {rc:Rc::new(t)}
    }
}

impl<T:Drawable> ImmutableShape<T> {
    pub fn translate(&self,x:f32,y:f32) -> Translate<Self> {
        Translate(self,x,y)
    }

    pub fn union<S:Shape>(&self,that:&S) -> Union<Self,S> {
        Union(self,that)
    }
}

impl<T> HasId for ImmutableShape<T> {
    fn id(&self) -> usize {
        Rc::downgrade(&self.rc).as_raw() as *const() as usize
    }
}

impl<T:Drawable> Drawable for ImmutableShape<T> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        self.rc.draw(canvas)
    }
}

impl<T:Drawable,S:Shape> std::ops::Add<&S> for &ImmutableShape<T> {
    type Output = Union<ImmutableShape<T>,S>;
    fn add(self, that:&S) -> Self::Output {
        self.union(that)
    }
}








define_compound_shapes! {
    Translate(child)(x:f32,y:f32)
    Union(child1,child2)()
}

impl<Child:Shape> Drawable for compound_shape_data::Translate<Child> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        canvas.translate(&self.x,&self.y);
        self.child.draw(canvas)
    }
}

impl<Child1:Shape,Child2:Shape> Drawable for compound_shape_data::Union<Child1,Child2> {
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
        let s1 = self.child1.draw(canvas);
        let s2 = self.child2.draw(canvas);
        canvas.union(s1,s2)
    }
}





pub mod shapes {
    pub use super::prim_shape::*;
    pub use super::compound_shape::*;
}


use shapes::*;







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

    Triangle(width:f32, height:f32) {
        vec2  norm = normalize(vec2(height,width/2.0));
        float dist = max(abs(position).x*norm.x + position.y*norm.y - height*norm.y, -position.y);
        return sdf(dist,bbox_center(width,height/2.0));
    }
}


pub fn main() {
    use shapes::*;

    let s1 = Circle(10.0);
//    let s2 = s1.translate(1.0,2.0);
    let s3 = &s1 + &s1;

    println!("{}", Canvas::run(&s3));
}


