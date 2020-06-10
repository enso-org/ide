//! This module contains the javascript base class for visualizations.

use crate::prelude::*;
use wasm_bindgen::prelude::*;

use ensogl::display::DomSymbol;
use super::PreprocessorCallback;
use wasm_bindgen::__rt::core::fmt::Formatter;

#[wasm_bindgen(module = "/src/component/visualization/foreign/java_script/visualization.js")]
extern "C" {

    #[allow(unsafe_code)]
    pub fn cls() -> JsValue;

    pub type Visualization;

    #[allow(unsafe_code)]
    #[wasm_bindgen(constructor)]
    fn new(init:VisualisationInitialisationData) -> Visualization;

    #[allow(unsafe_code)]
    #[wasm_bindgen(method)]
    fn setPreprocessor(this: &Visualization);
}


/// Data that is passed into the javascript Visualization baseclass.
#[allow(missing_docs)]
#[wasm_bindgen]
pub struct VisualisationInitialisationData {
    #[wasm_bindgen(skip)]
    pub root             : DomSymbol,

    #[wasm_bindgen(skip)]
    pub set_preprocessor : Box<dyn PreprocessorCallback>,
}

impl Debug for VisualisationInitialisationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"VisualisationInitialisationData({:?})", &self.root)
    }
}

impl VisualisationInitialisationData {
    /// Constructor.
    pub fn new<F: PreprocessorCallback + 'static>(root:DomSymbol, closure:F) -> Self {
        let set_preprocessor:Box<dyn PreprocessorCallback> = Box::new(closure);
        VisualisationInitialisationData{
            root,
            set_preprocessor,
        }
    }
}

#[wasm_bindgen]
impl VisualisationInitialisationData {
    /// Getter for the root element for the visualization.
    pub fn root(&self) -> JsValue {
        self.root.dom().into()
    }

    /// Helper method to emit an preprocessor change event from the visualisation.
    pub fn emit_preprocessor_change(&self, code:String){
        let closure = &self.set_preprocessor;
        (*closure)(code);
    }
}
