//! This module contains the javascript base class for visualizations.

use wasm_bindgen::prelude::*;


#[wasm_bindgen(module = "/src/component/visualization/foreign/java_script/visualization.js")]
extern "C" {

    #[allow(unsafe_code)]
    pub fn cls() -> JsValue;

    pub type Visualization;

    #[allow(unsafe_code)]
    #[wasm_bindgen(constructor)]
    fn new(root:JsValue) -> Visualization;

    #[allow(unsafe_code)]
    #[wasm_bindgen(method)]
    fn setPreprocessor(this: &Visualization);
}
