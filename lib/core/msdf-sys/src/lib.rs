extern crate wasm_bindgen;
#[macro_use] extern crate slice_as_array;

use wasm_bindgen::prelude::*;
use js_sys::*;

const MAX_MSDF_SIZE : usize = 32;
pub const MSDF_CHANNELS_COUNT : usize = 3;
const MSDF_DATA_SIZE : usize = MAX_MSDF_SIZE*MAX_MSDF_SIZE*MSDF_CHANNELS_COUNT;

pub struct FontHandle {
    handle: JsValue
}

pub struct MSDF {
    pub width  : usize,
    pub height : usize,
    pub data   : [f32;MSDF_DATA_SIZE]
}

pub struct MSDFParameters {
    width                         : usize,
    height                        : usize,
    edge_coloring_angle_threshold : f64,
    range                         : f64,
    scale                         : (f64, f64),
    translate                     : (f64, f64),
    edge_threshold                : f64,
    overlap_support               : bool
}

#[wasm_bindgen(module = "msdfgen_wasm.js")]
extern {

fn ccall(
    name        : &str,
    return_type : &str,
    types       : js_sys::Array,
    values      : js_sys::Array
) -> JsValue;

fn getValue(address: usize, a_type: &str) -> JsValue;

fn _msdfgen_maxMSDFSize() -> usize;

fn _msdfgen_generateMSDF(
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

fn _msdfgen_freeFont(font_handle: JsValue);

}

fn make_js_array_from_data_view(data: &[u8]) -> Uint8Array {
    unsafe {
        return Uint8Array::view(data)
    }
}

pub fn load_font_memory(data: &[u8]) -> FontHandle {

    let param_types = js_sys::Array::new_with_length(2);
    param_types.set(0, JsValue::from_str("array"));
    param_types.set(1, JsValue::from_str("number"));
    let params = js_sys::Array::new_with_length(2);
    params.set(0, JsValue::from(make_js_array_from_data_view(data)));
    params.set(1, JsValue::from_f64(data.len() as f64));
    let handle = ccall(
        "msdfgen_loadFontMemory",
        "number",
        param_types,
        params);
    FontHandle { handle }

}

impl Drop for FontHandle {
    fn drop(&mut self) {
        _msdfgen_freeFont(self.handle.clone())
    }
}

pub fn generate_msdf(
    font    : &FontHandle,
    unicode : u32,
    params  : MSDFParameters
) -> MSDF {
    assert!(params.width <= MAX_MSDF_SIZE);
    assert!(params.height <= MAX_MSDF_SIZE);

    let output_address = _msdfgen_generateMSDF(
        params.width,
        params.height,
        font.handle.clone(),
        unicode,
        params.edge_coloring_angle_threshold,
        params.range,
        params.scale.0,
        params.scale.1,
        params.translate.0,
        params.translate.1,
        params.edge_threshold,
        params.overlap_support
    );
    let mut data : [f32;MSDF_DATA_SIZE] = [0.0;MSDF_DATA_SIZE];
    for i in 0..params.width*params.height*MSDF_CHANNELS_COUNT {
        data[i] = getValue(output_address + i*4, "float")
            .as_f64()
            .unwrap() as f32
    }

    MSDF { width: params.width, height: params.height, data }
}

#[cfg(test)]
mod tests {
    extern crate wasm_bindgen_test;
    use wasm_bindgen_test::*;
    use crate::*;

    #[wasm_bindgen_test]
    fn generate_msdf_for_capital_a() {
        let test_font : &[u8] = include_bytes!("DejaVuSansMono-Bold.ttf");
        let expected_output_raw : &[u8] = include_bytes!("output.bin");
        let font = load_font_memory(test_font);
        let params = MSDFParameters {
            width: 32,
            height: 32,
            edge_coloring_angle_threshold: 3.0,
            range: 2.0,
            scale: (1.0, 1.0),
            translate: (0.0, 0.0),
            edge_threshold: 1.001,
            overlap_support: true
        };
        let msdf = generate_msdf(&font, 'A' as u32, params);
        for i in 0..(32*32*3) {
            let expected_f = f32::from_le_bytes(
                slice_to_array_clone!(
                    &expected_output_raw[i*4..(i+1)*4],
                    [u8; 4]
                ).unwrap()
            );
            assert_eq!(expected_f, msdf.data[i], "Index {}", i);
        }
    }

    #[wasm_bindgen_test]
    fn msdf_data_limits() {
        assert!(MAX_MSDF_SIZE < _msdfgen_maxMSDFSize());
    }
}
