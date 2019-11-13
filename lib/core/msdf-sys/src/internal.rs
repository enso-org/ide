use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen(module = "msdfgen_wasm.js")]
extern {
    // Function provided by emscripten sdk,
    // for details, search for "emscripten call c from js"
    pub fn ccall( // Note [extern function names]
        name        : &str,
        return_type : &str,
        types       : js_sys::Array,
        values      : js_sys::Array
    ) -> JsValue;

    // function provided by emscripten sdk,
    // for details, search for "emscripten call c from js"
    pub fn getValue(address: usize, a_type: &str) -> JsValue;
    // Note [extern function names]

    pub fn _msdfgen_maxMSDFSize() -> usize; // Note [extern function names]

    pub fn _msdfgen_generateMSDF( // Note [extern function names]
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
    // Note [extern function names]

    /* Note [extern function names]
     * Functions declared in this module are exported directly from
     * c library or from emscripten sdk. Therefore, they does not fulfill
     * the rust function naming guidelines
     */
}

pub mod emscripten_data_types {
    pub const FLOAT_SIZE_IN_BYTES : usize = 4;

    pub const ARRAY  : &str = "array";
    pub const NUMBER : &str = "number";
    pub const FLOAT  : &str = "float";
}

pub fn copy_f32_data_from_msdfgen_memory(
    address        : usize,
    output         : &mut[f32],
    elements_count : usize
) {
    for (i, element) in
        output.iter_mut().enumerate().take(elements_count) {

        let offset = i * emscripten_data_types::FLOAT_SIZE_IN_BYTES;
        *element = getValue(
            address + offset,
            emscripten_data_types::FLOAT
        ).as_f64().unwrap() as f32;
    }
}