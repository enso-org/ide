//! Canvas for drawing vector graphics. See the documentation of `Canvas` to learn more.

use crate::prelude::*;

use super::item::GlslItem;



// ================
// === Drawable ===
// ================

/// Describes every shape which can be painted on the canvas.
pub trait Drawable {
    /// Draw the element on the canvas.
    fn draw(&self, canvas:&mut Canvas) -> CanvasShape;
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
    pub fn get_new_id(&mut self) -> usize {
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
    pub fn add_current_function_code_line<S:Str>(&mut self, line:S) {
        self.current_function_lines.push(format!("    {}",line.as_ref()));
    }

    /// Defines a new variable in the GLSL code.
    pub fn define<E:Str>(&mut self, ty:&str, name:&str, expr:E) {
        let max_type_length = 6;
        let max_name_length = 13;
        let ty              = format!("{:1$}" , ty   , max_type_length);
        let name            = format!("{:1$}" , name , max_name_length);
        self.add_current_function_code_line(iformat!("{ty} {name} = {expr.as_ref()};"));
    }

    /// Submits the `current_function_lines` as a new shape construction function in the GLSL code.
    pub fn submit_shape_constructor(&mut self, name:&str) {
        let body = self.current_function_lines.join("\n");
        let func = iformat!("shape {name} (globals global, vec2 position) {{\n{body}\n}}");
        self.current_function_lines = default();
        self.functions.push(func);
    }

    /// Get the final GLSL code.
    pub fn to_glsl(&self) -> String {
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
    pub fn new_canvas_shape(&mut self) -> CanvasShape {
        let num = self.get_new_shape_num();
        CanvasShape::new(num)
    }

    /// Defines a new shape with a new id and associated parameters, like color.
    pub fn define_shape(&mut self, sdf:&str) -> CanvasShape {
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

