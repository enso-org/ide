//! This module contains GLSL code builder. It allows translating complex vector shapes to the GLSL
//! code.

use crate::prelude::*;

use super::canvas::Canvas;
use super::super::class::Shape;
use crate::display::shape::primitive::def::sdf;
use crate::display::symbol::shader::builder::CodeTemplete;
use crate::display::shape::primitive::shader::overload;

const HELPERS         :&str = include_str!("../glsl/helpers.glsl");
const FRAGMENT_RUNNER :&str = include_str!("../glsl/fragment_runner.glsl");


pub fn header(label:&str) -> String {
    let border_len = label.len() + 8;
    let border     = "=".repeat(border_len);
    iformat!("// {border}\n// === {label} ===\n// {border}")
}

/// GLSL code builder.
pub struct Builder {}

impl Builder {
    /// Returns the final GLSL code.
    pub fn run<S:Shape>(shape:&S) -> CodeTemplete {
        let sdf_defs     = sdf::all_shapes_glsl_definitions();
        let mut canvas   = Canvas::default();
        let shape_ref    = shape.draw(&mut canvas);
        let defs_header  = header("SDF Primitives");
        let shape_header = header("Shape Definition");
        canvas.add_current_function_code_line(iformat!("return {shape_ref.getter()};"));
        canvas.submit_shape_constructor("run");
        let defs = iformat!("{defs_header}\n\n{sdf_defs}\n\n\n\n{shape_header}\n\n{canvas.to_glsl()}");
//        CodeTemplete::new(HELPERS.to_string(),FRAGMENT_RUNNER.to_string(),default())

        let redirections = overload::builtin_redirections();
        let helpers      = overload::allow_overloading(&HELPERS.to_string());

        let defs = overload::allow_overloading(&defs);

        println!("{}",defs);

//        let helpers = format!("{}\n\n{}",redirections,helpers);
        let helpers = format!("{}\n\n{}\n\n{}",redirections,helpers,defs);

        CodeTemplete::new(helpers,FRAGMENT_RUNNER.to_string(),default())
    }
}

