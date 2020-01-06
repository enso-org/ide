use crate::prelude::*;

use wasm_bindgen::prelude::*;


#[wasm_bindgen(module = "/src/display/shape/primitive/glsl/overload.js")]
extern "C" {
    pub fn builtin_redirections() -> String;
    pub fn allow_overloading(s:&str) -> String;
}