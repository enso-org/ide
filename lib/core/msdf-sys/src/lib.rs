extern crate wasm_bindgen;

mod internal;

use internal::{
    ccall,
    getValue,
    _msdfgen_generateMSDF,
    _msdfgen_freeFont,
    emscripten_data_types
};
use wasm_bindgen::JsValue;
use js_sys::Uint8Array;

// ==================
// === FontHandle ===
// ==================

pub struct FontHandle {
    handle: JsValue
}

pub fn load_font_memory(data: &[u8]) -> FontHandle {

    let param_types = js_sys::Array::new_with_length(2);
    param_types.set(0, JsValue::from_str(emscripten_data_types::ARRAY));
    param_types.set(1, emscripten_data_types::NUMBER.into());
    let params = js_sys::Array::new_with_length(2);
    unsafe { // Note [Usage of Uint8Array::view
        params.set(0, JsValue::from(Uint8Array::view(data)));
    }
    params.set(1, JsValue::from_f64(data.len() as f64));
    let handle = ccall(
        "msdfgen_loadFontMemory",
        emscripten_data_types::NUMBER,
        param_types,
        params);
    FontHandle { handle }
}

/*
 * Note [Usage of Uint8Array::view]
 * ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
 * We use view in this place to avoid copying font data. This is the only way
 * to do it with js_sys structures. The Uint8Array does not leave function
 * scope, so does not excess lifetime of data
 */

impl Drop for FontHandle {
    fn drop(&mut self) {
        _msdfgen_freeFont(self.handle.clone())
    }
}

// =======================================
// === MutlichannelSignedDistanceField ===
// =======================================

pub struct MutlichannelSignedDistanceField {
    pub width  : usize,
    pub height : usize,
    pub data   : [f32; Self::MAX_DATA_SIZE]
}

pub struct MSDFParameters {
    pub width                         : usize,
    pub height                        : usize,
    pub edge_coloring_angle_threshold : f64,
    pub range                         : f64,
    pub scale                         : (f64, f64),
    pub translate                     : (f64, f64),
    pub edge_threshold                : f64,
    pub overlap_support               : bool
}

impl MutlichannelSignedDistanceField {
    pub const MAX_SIZE       : usize = 32;
    pub const CHANNELS_COUNT : usize = 3;
    pub const MAX_DATA_SIZE  : usize = Self::MAX_SIZE * Self::MAX_SIZE *
        Self::CHANNELS_COUNT;

    pub fn generate(
        font    : &FontHandle,
        unicode : u32,
        params  : MSDFParameters
    ) -> MutlichannelSignedDistanceField {
        assert!(params.width  <= Self::MAX_SIZE);
        assert!(params.height <= Self::MAX_SIZE);

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
        let mut data : [f32; Self::MAX_DATA_SIZE] = [0.0; Self::MAX_DATA_SIZE];
        let data_size = params.width*params.height*Self::CHANNELS_COUNT;

        for (i, data_element) in
            data.iter_mut().enumerate().take(data_size) {

            let offset = i * emscripten_data_types::FLOAT_SIZE_IN_BYTES;
            *data_element = getValue(
                output_address + offset,
                emscripten_data_types::FLOAT
            ).as_f64().unwrap() as f32;
        }

        MutlichannelSignedDistanceField {
            width: params.width,
            height: params.height,
            data
        }
    }
}



#[cfg(test)]
mod tests {
    extern crate wasm_bindgen_test;
    extern crate slice_as_array;
    use wasm_bindgen_test::wasm_bindgen_test;
    use slice_as_array::slice_to_array_clone;
    use internal::_msdfgen_maxMSDFSize;
    use crate::*;

    #[wasm_bindgen_test]
    fn generate_msdf_for_capital_a() {
        // given
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
        // when
        let msdf = MutlichannelSignedDistanceField::generate(
            &font,
            'A' as u32,
            params
        );
        // then
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
        assert!(MutlichannelSignedDistanceField::MAX_SIZE <
            _msdfgen_maxMSDFSize());
    }
}
