use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen(module = "msdfgen_wasm.js")]
extern {
    // Function provided by emscripten sdk,
    // for details, search for "emscripten call c from js" in your internet
    pub fn ccall(
        name        : &str,
        return_type : &str,
        types       : js_sys::Array,
        values      : js_sys::Array
    ) -> JsValue;

    // function provided by emscripten sdk,
    // for details, search for "emscripten call c from js" in your internet
    pub fn getValue(address: usize, a_type: &str) -> JsValue;

    pub fn _msdfgen_maxMSDFSize() -> usize;

    pub fn _msdfgen_generateMSDF(
        width                           : usize,
        height                          : usize,
        font_handle                     : JsValue,
        unicode                         : u32,
        edge_coloring_angle_threshold   : f64,
        range                           : f64,
        scale_x                         : f64,
        scale_y                         : f64,
        translate_x                     : f64,
        translate_y                     : f64,
        edge_threshold                  : f64,
        overlap_support                 : bool
    ) -> usize;

    pub fn _msdfgen_freeFont(font_handle: JsValue);
}

pub mod emscripten_data_types {
    pub const FLOAT_SIZE_IN_BYTES : usize = 4;

    pub const ARRAY  : &str = "array";
    pub const NUMBER : &str = "number";
    pub const FLOAT  : &str = "float";
}