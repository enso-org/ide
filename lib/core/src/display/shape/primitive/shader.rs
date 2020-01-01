//! Shader primitives used to render vector shapes on GPU.

use crate::prelude::*;

use std::include_str;
use inflector::Inflector;
use crate::display::symbol::geometry::primitive::mesh::buffer::item::Item;
use nalgebra::Vector2;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;


const GLSL_DEFS:&str = include_str!("shader/defs.glsl");


fn mk_bb_name    <S:Str> (s:S) -> String { format!("{}_bb"    , s.as_ref()) }
fn mk_id_name    <S:Str> (s:S) -> String { format!("{}_id"    , s.as_ref()) }
fn mk_cd_name    <S:Str> (s:S) -> String { format!("{}_cd"    , s.as_ref()) }
fn mk_sdf_name   <S:Str> (s:S) -> String { format!("{}_sdf"   , s.as_ref()) }
fn mk_shape_name <S:Str> (s:S) -> String { format!("shape_{}" , s.as_ref()) }

//defCdC = Color.rgb [1,0,0,1]
//defCd  = "rgb2lch(#{GLSL.toCode defCdC})"

// ==================
// === SDF Canvas ===
// ==================

#[derive(Clone,Debug)]
pub struct CanvasShape {
    shape_num : usize,
    ids       : Vec<usize>,
    name      : String,
}

impl CanvasShape {
    pub fn new(shape_num:usize) -> Self {
        let ids  = default();
        let name = mk_shape_name(shape_num.to_string());
        Self {shape_num,ids,name}
    }

    pub fn add_id(&mut self, id:usize) {
        self.ids.push(id);
    }

    pub fn add_ids(&mut self, ids:&Vec<usize>) {
        self.ids.extend(ids)
    }
}


enum IdGenerationType {

}

pub trait Glsl {
    fn is_zero (&self) -> bool;
    fn glsl    (&self) -> String;
}

impl Glsl for str {
    fn is_zero (&self) -> bool   { self == "0" || self == "0.0" }
    fn glsl    (&self) -> String { self.into() }
}

impl Glsl for &str {
    fn is_zero (&self) -> bool   { (*self).is_zero() }
    fn glsl    (&self) -> String { (*self).glsl()    }
}

impl Glsl for f32 {
    fn is_zero (&self) -> bool   { *self == 0.0 }
    fn glsl    (&self) -> String {
        let is_int = self.fract() == 0.0;
        if is_int { iformat!("{self}.0") }
        else      { iformat!("{self}")   }
    }
}



#[derive(Debug,Default)]
pub struct Canvas {
    shape_num  : usize,
    last_id    : usize,
    bb_lines   : Vec<String>,
    code_lines : Vec<String>,
}

impl Canvas {
    pub fn get_new_id(&mut self) -> usize {
        let id = self.last_id;
        self.last_id += 1;
        id
    }

    pub fn add_code_line(&mut self, line:String) {
        self.code_lines.push(line);
    }

    pub fn add_code_line_ind(&mut self, line:String) {
        self.add_code_line(format!("    {}",line));
    }

    pub fn define<E:Str>(&mut self, ty:&str, name:&str, expr:E) {
        let max_type_length = 7;
        let max_name_length = 13;
        let ty              = format!("{:1$}" , ty   , max_type_length);
        let name            = format!("{:1$}" , name , max_name_length);
        self.add_code_line_ind(iformat!("{ty} {name} = {expr.as_ref()};"));
    }

    pub fn add_bb_line(&mut self, line:String) {
        self.bb_lines.push(line);
    }

    pub fn gen_new_color_id(&mut self, name:&str) -> usize {
        let id       = self.get_new_id();
        let id_name  = mk_id_name(name);
        let sdf_name = mk_sdf_name(name);
        self.define("id",&id_name,iformat!("new_id_layer({sdf_name},{id})"));
        id
    }

    pub fn merge_id_layers(&mut self, a:CanvasShape, b:CanvasShape, name:String) {
        let id_name = mk_id_name(&name);
        self.define("id",&id_name,iformat!("id_union({a.name},{b.name},{a.name}.id,{b.name}.id)"));
    }

    pub fn intersect_id_layers(&mut self, a:CanvasShape, b:CanvasShape, name:String) {
        let id_name = mk_id_name(&name);
        self.define("id",&id_name,iformat!("id_intersection({a.name},{b.name},{a.name}.id)"));
    }

    pub fn diff_id_layers(&mut self, a:CanvasShape, b:CanvasShape, name:String) {
        let id_name = mk_id_name(&name);
        self.define("id",&id_name,iformat!("id_difference({a.name},{b.name},{a.name}.id)"));
    }

    pub fn keep_id_layer(&mut self, a:CanvasShape, name:String) {
        let id_name = mk_id_name(&name);
        self.define("id",&id_name,iformat!("{a.name}.id"));
    }

    pub fn code(&self) -> String {
        self.code_lines.join("\n")
    }

    pub fn get_new_shape_num(&mut self) -> usize {
        let out = self.shape_num;
        self.shape_num += 1;
        out
    }

    pub fn register_shape(&mut self) -> CanvasShape {
        let num = self.get_new_shape_num();
        CanvasShape::new(num)
    }

    pub fn new_shape(&mut self, sdf:&str, cd:Option<&str>) -> CanvasShape {
        let color     = "rgb2lch(vec3(1.0,0.0,0.0)";
        let mut shape = self.register_shape();
        let id_name   = mk_id_name  (&shape.name);
        let cd_name   = mk_cd_name  (&shape.name);
        let sdf_name  = mk_sdf_name (&shape.name);
        self.define("color" , &cd_name    , iformat!("{color}"));
        self.define("sdf"   , &sdf_name   , iformat!("{sdf}"));
        shape.add_id(self.gen_new_color_id(&shape.name));
        self.define("shape", &shape.name, iformat!("shape({id_name},{cd_name},{sdf_name})"));
        shape
    }

    pub fn new_shape_from_expr(&mut self, expr:&str) -> CanvasShape {
        let shape = self.register_shape();
        self.define("shape",&shape.name,expr);
        shape
    }

    pub fn union(&mut self, s1:CanvasShape, s2:CanvasShape) -> CanvasShape {
        let mut shape = self.new_shape_from_expr(&iformat!("union({s1.name},{s2.name})"));
        shape.add_ids(&s1.ids);
        shape.add_ids(&s2.ids);
        shape
    }


    pub fn mv(&mut self, x:f32, y:f32) {
        self.add_code_line_ind(iformat!("position = sdf_translate(position, vec2({x.glsl()},{y.glsl()}));"));
    }

}


static GLOBAL_SHAPE_COUNT: AtomicUsize = AtomicUsize::new(0);



pub trait SdfShape {
    fn sdf_code(&self) -> String;
}

macro_rules! shapes {
    ( $($name:ident $args:tt $body:tt)* ) => {
        $(shape! {$name $args $body} )*
    };
}

macro_rules! shape {
    ( $name:ident ( $($field:ident : $field_type:ty),* $(,)? ) { $($code:tt)* } ) => {
        /// The shape definition.
        #[derive(Debug,Clone)]
        pub struct $name {
            pub id        : usize,
            pub glsl_name : String,
            $(pub $field  : String),*
        }

        /// Smart shape constructor.


        impl $name {
            /// Constructor.
            pub fn new <$($field:Glsl),*> ( $($field : $field),* ) -> Self {
                let id        = GLOBAL_SHAPE_COUNT.fetch_add(1, Ordering::Relaxed);
                let glsl_name = stringify!($name).to_snake_case();
                $(let $field = $field.glsl();)*
                Self {id,glsl_name,$($field),*}
            }

            /// Draws the shape on the provided canvas. Shapes are always drawn in the center of
            /// the canvas. In order to move them somewhere, use the canvas moving API.
            pub fn draw(&self, canvas:&mut Canvas) -> CanvasShape {
                let args = vec!["position", $(stringify!($field)),* ].join(",");
                let code = format!("{}({})",self.glsl_name,args);
                canvas.new_shape(&code,None)
            }
        }

        impl SdfShape for $name {
            fn sdf_code(&self) -> String {
                let body = stringify!($($code)*);
                let args = vec!["vec2 position".to_string(), $(
                    format!("{} {}", <$field_type as Item>::gpu_type_name(), stringify!($field))
                ),*].join(", ");
                format!("sdf {} ({}) {{ {} }}",self.glsl_name,args,body)
            }
        }

        impl Shape for $name {
            fn id(&self) -> usize {
                self.id
            }

            fn render_glsl(&self, renderer:&mut GlslRenderer) -> CanvasShape {
                self.draw(&mut renderer.canvas)
            }
        }
    };
}



trait Shape {
    fn id(&self) -> usize {
        0
    }

    fn render_glsl(&self, renderer:&mut GlslRenderer) -> CanvasShape {
        unimplemented!()
    }
}






pub struct Move<S> {
    shape : S,
    x     : f32,
    y     : f32,
}

impl<S> Move<S> {
    pub fn new(shape:S,x:f32,y:f32) -> Self {
        Self {shape,x,y}
    }
}

impl<S:Shape> Shape for Move<S> {
    fn render_glsl(&self, renderer:&mut GlslRenderer) -> CanvasShape {
        renderer.with_new_tx_ctx(|r| {
            r.canvas.mv(self.x,self.y);
            r.render_shape(&self.shape)
        })
    }
}


//export class Move extends Shape
//constructor: (@a, @x, @y) -> super(); @addChildren @a
//renderGLSL: (r) ->
//r_x = resolve r, @x
//r_y = resolve r, @y
//r.withNewTxCtx () =>
//r.canvas.move r_x, r_y
//r.renderShape @a




#[derive(Debug,Default)]
struct GlslRenderer {
    canvas      : Canvas,
    done        : HashMap<(usize,usize), CanvasShape>,
    tx_ctx      : usize,
    last_tx_ctx : usize,
}

impl GlslRenderer {
    pub fn get_new_tx_ctx(&mut self) -> usize {
        self.last_tx_ctx += 1;
        self.last_tx_ctx
    }

    pub fn with_new_tx_ctx<F:FnOnce(&mut Self)->T,T>(&mut self, f:F) -> T {
        let old_ctx = self.tx_ctx;
        let new_ctx = self.get_new_tx_ctx();
        self.tx_ctx = new_ctx;
        self.canvas.add_code_line_ind(iformat!("vec2 position_{new_ctx} = position;"));
        let out = f(self);
        self.canvas.add_code_line_ind(iformat!("position = position_{new_ctx};"));
        self.tx_ctx = old_ctx;
        out
    }

    pub fn render_shape<S:Shape>(&mut self, shape:&S) -> CanvasShape {
        let shape_ptr    = shape.id();
        let canvas_shape = self.done.get(&(shape_ptr,self.tx_ctx));
        match canvas_shape {
            Some(s) => s.clone(),
            None    => {
                let canvas_shape = shape.render_glsl(self);
                self.done.insert((shape_ptr,self.tx_ctx), canvas_shape.clone());
                canvas_shape
            }
        }
    }

    pub fn render<S:Shape>(&mut self, shape:&S) -> String {
        let canvas_shape = self.render_shape(shape);
        iformat!("shape main(vec2 position) {{\n{self.canvas.code()}\n    return {canvas_shape.name};\n}}")
    }
}


pub trait ShapeOps
where Self:Sized+Clone {
    fn mv(&self,x:f32,y:f32) -> Move<Self> {
        Move::new(self.clone(),x,y)
    }
}

impl<T> ShapeOps for T where T:Shape+Clone {}





shapes! {

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
    let mut r:GlslRenderer = default();
    let canvas = &mut r.canvas;

//    let c1 = Circle::new("10.0");
//    let c2 = Circle::new("10.0");
//    let s1 = c1.draw(canvas);
//    let s2 = c2.draw(canvas);
//    canvas.union(s1,s2);

    let s1 = Circle::new(10.0).mv(1.0,2.0);

    println!("{}", r.render(&s1));

//
//    println!("{}", c1.sdf_code());
}


