//! This module contains the javascript base class for visualizations.

use crate::prelude::*;

use crate::component::visualization::foreign::java_script::PreprocessorCallback;
use crate::component::visualization::instance::PreprocessorConfiguration;
use crate::component::type_coloring;
use crate::Type;
use crate::display::style;

use ensogl::display::DomSymbol;
use ensogl::display::shape::StyleWatch;
use ensogl::data::color;
use fmt::Formatter;
use wasm_bindgen::prelude::*;
use web_sys::HtmlDivElement;



// =================
// === Constants ===
// =================

/// Name of the visualization base class in JavaScript sources.
pub const JS_CLASS_NAME : &str = "Visualization";



// ===========================
// === JavaScript Bindings ===
// ===========================

#[wasm_bindgen(module="/src/component/visualization/foreign/java_script/visualization.js")]
extern "C" {
    #[allow(unsafe_code)]
    fn __Visualization__() -> JsValue;

    #[allow(unsafe_code)]
    #[wasm_bindgen(extends = js_sys::Object)]
    pub type Visualization;

    #[allow(unsafe_code)]
    #[wasm_bindgen(constructor)]
    fn new(init:JsConsArgs) -> Visualization;

    #[allow(unsafe_code)]
    #[wasm_bindgen(catch, js_name = __emitPreprocessorChange__, method)]
    pub fn emitPreprocessorChange(this:&Visualization) -> Result<(),JsValue>;
}

/// Provides reference to the visualizations JavaScript base class.
pub fn js_class() -> JsValue {
    __Visualization__()
}



// =====================
// === Rust Bindings ===
// =====================

/// Data that is passed into the javascript Visualization baseclass.
#[allow(missing_docs)]
#[wasm_bindgen]
pub struct JsConsArgs {
    #[wasm_bindgen(skip)]
    pub root : HtmlDivElement,
    theme : JsTheme,
    #[wasm_bindgen(skip)]
    pub set_preprocessor : Box<dyn PreprocessorCallback>,
}

impl Debug for JsConsArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"JsConsArgs({:?})", &self.root)
    }
}

impl JsConsArgs {
    /// Constructor.
    pub fn new<F:'static+PreprocessorCallback>
    (root:DomSymbol, styles:StyleWatch, closure:F) -> Self {
        let set_preprocessor = Box::new(closure);
        let theme = JsTheme {styles};
        let root = root.dom().clone();
        JsConsArgs {root,theme,set_preprocessor}
    }
}

#[wasm_bindgen]
impl JsConsArgs {
    /// Getter for the root element for the visualization.
    pub fn root(&self) -> JsValue {
        self.root.clone().into()
    }

    /// Getter for the theming API that we expose to JS visualizations
    pub fn theme(&self) -> JsTheme {
        self.theme.clone().into()
    }

    /// Helper method to emit an preprocessor change event from the visualisation.
    pub fn emit_preprocessor_change(&self, code:Option<String>, module:Option<String>){
        let closure             = &self.set_preprocessor;
        let preprocessor_config = PreprocessorConfiguration::from_options(code,module);
        (*closure)(preprocessor_config);
    }
}

/// The theming API that we expose to JS visualizations
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct JsTheme {
    styles: StyleWatch
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl JsTheme {
    /// Takes a qualified type name and returns the color that is used in the GUI for that type.
    pub fn getColorForType(&self, tp_name: &str) -> String {
        let tp = Type::from(tp_name.to_string());
        let lcha = type_coloring::compute(&tp,&self.styles);
        format_lcha(lcha)
    }

    /// Takes a qualified type name and returns the color that should be used for foreground
    /// (e.g. text) that is shown on top of the background color returned by getColorForType.
    pub fn getForegroundColorForType(&self, _tp_name: &str) -> String {
        "white".to_string()
    }

    /// Queries style sheet value for a value.
    pub fn get(&self, path: &str) -> Option<String> {
        if let style::Data::Color(lcha) = self.styles.get(path)? {
            Some(format_lcha(lcha))
        } else {
            None
        }
    }
}

fn format_lcha(lcha: color::Lcha) -> String {
    let rgba = color::Rgba::from(lcha);
    format!("rgba({:.0}, {:.0}, {:.0}, {})",
        (rgba.red * 255.0).round(),
        (rgba.green * 255.0).round(),
        (rgba.blue * 255.0).round(),
        rgba.alpha)
}
