//! This module contains GLSL code builder. It allows translating complex vector shapes to the GLSL
//! code.

use crate::prelude::*;

use super::canvas::Canvas;
use super::super::class::Shape;
use crate::display::shape::primitive::def::sdf;


//const GLSL_DEFS:&str = include_str!("helpers.glsl");


pub fn header(label:&str) -> String {
    let border_len = label.len() + 8;
    let border     = "=".repeat(border_len);
    iformat!("// {border}\n// === {label} ===\n// {border}")
}

/// GLSL code builder.
pub struct Builder {}

impl Builder {
    /// Returns the final GLSL code.
    pub fn run<S:Shape>(shape:&S) -> String {
        let sdf_defs     = sdf::all_shapes_glsl_definitions();
        let mut canvas   = Canvas::default();
        let shape_ref    = shape.draw(&mut canvas);
        let defs_header  = header("SDF Primitives");
        let shape_header = header("Shape Definition");
        canvas.add_current_function_code_line(iformat!("return {shape_ref.getter()};"));
        canvas.submit_shape_constructor("run");
        iformat!("{defs_header}\n\n{sdf_defs}\n\n\n\n{shape_header}\n\n{canvas.to_glsl()}")
    }
}
