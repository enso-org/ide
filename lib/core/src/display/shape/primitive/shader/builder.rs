//! This module contains GLSL code builder. It allows translating complex vector shapes to the GLSL
//! code.

use crate::prelude::*;

use super::canvas::Canvas;
use super::super::class::Shape;



//const GLSL_DEFS:&str = include_str!("helpers.glsl");



/// GLSL code builder.
pub struct Builder {}

impl Builder {
    /// Returns the final GLSL code.
    pub fn run<S:Shape>(shape:&S) -> String {
        let mut canvas = Canvas::default();
        let shape_ref  = shape.draw(&mut canvas);
        canvas.add_current_function_code_line(iformat!("return {shape_ref.getter()};"));
        canvas.submit_shape_constructor("run");
        canvas.to_glsl()
    }
}


