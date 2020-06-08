//! This module contains functionality to create a `Class` object from a JS source strings.
//!
//! The JS source needs to follow the following spec:
//! * the provided code needs to be the body of a function that takes no arguments and returns a
//!   class object.
//! * the returned class MAY define a static field `label` that will be read once when processing
//!   the class. The label value string representation will be used as a a textual representation
//!   of the visualization, e.g., in selection forms or widget headings. If this field is not set a
//!   default value will be used.
//! * the returned class MAY define a static field `inputType` that will be read once when
//!   processing the class. The string representation of the `inputType` value will be interpreted
//!   as an Enso type. This type will be used to determine what data the visualization can receive.
//!   If this field is not set the type "Any" will be assumed by default.
//! * the returned class object will be instantiated by calling its constructor with no arguments.
//! * the instantiated object MAY define a method `onDataReceived(root, data)`. If this method is
//!   present, it will be called whenever no data is provided for the visualization. The argument
//!   `root` will be set to the DOM element that should be used as the parent of the visualization.
//!   The argument `data` will be set to the incoming data, which may be in any format that is
//!   compatible with the type specified in `inputType`. If this method is not present, not data
//!   updates are provided.
//! * the instantiated object CAN define a method `setSize(root, size)`. If this method is
//!   present, it will be called whenever the parent of the visualisation changes it's size. If
//!   this method is not present, not size updates are provided.
//!
//! TODO: this should be read by someone with deep knowledge of JS to ensure it all makes sense and
//!       leaves no corner cases.
//! TODO: Thoughts on changing spec:
//!        * should root be provided once in the constructor?
//!        * could setSize be removed and instead the visualisation should change it's size based
//!          on the size of the root node?
//!
//! Example
//! ------
//! ```JS
//! class Visualization {
//!         static label     = "plot chart"
//!         static inputType = "Any"
//!         onDataReceived(root, data) {}
//!         setSize(root, size) {}
//!     }
//!     return Visualizations;
//!```


// TODO: write detailed specification. Please note that EVERYTHING should be optional. Make sure it
//       is handled propelry in all places in the code. add tests of visualizations without fields.
//     class Visualization {
//         static label     = "plot chart"
//         static inputType = "Any"
//         onDataReceived(root, data) {}
//         setSize(root, size) {}
//     }
//     return Visualizations;

use crate::prelude::*;

use crate::component::visualization::InstantiationResult;
use crate::component::visualization::InstantiationError;
use crate::component::visualization;
use crate::data::*;

use super::instance::Instance;

use ensogl::display::Scene;
use ensogl::system::web::JsValue;
use js_sys::JsString;
use js_sys;
use wasm_bindgen::JsCast;



// =================
// === Constants ===
// =================

const LABEL_FIELD      : &str = "label";
const INPUT_TYPE_FIELD : &str = "inputType";



// ==================
// === Definition ===
// ==================

/// JavaScript visualization definition.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Definition {
    class     : JsValue,
    signature : visualization::Signature,
}

impl Definition {
    /// Create a visualization source from piece of JS source code. Signature needs to be inferred.
    pub fn new (library:impl Into<LibraryName>, source:impl AsRef<str>) -> Result<Self,Error> {
        let source     = source.as_ref();
        let context    = JsValue::NULL;
        let function   = js_sys::Function::new_no_args(source);
        let class      = function.call0(&context).map_err(Error::InvalidFunction)?;

        let library    = library.into();
        let input_type = try_str_field(&class,INPUT_TYPE_FIELD).unwrap_or_default();
        let label      = label(&class)?;
        let path       = visualization::Path::new(library,label);
        let signature  = visualization::Signature::new(path,input_type);

        Ok(Self{class,signature})
    }

    fn new_instance(&self, scene:&Scene) -> InstantiationResult {
        let js_new  = js_sys::Function::new_with_args("cls", "return new cls()");
        let context = JsValue::NULL;
        let obj     = js_new.call1(&context,&self.class).map_err(InstantiationError::ConstructorError)?;
        let instance = Instance::from_object(obj).unwrap(); // ?; FIXME
        instance.set_dom_layer(&scene.dom.layers.main);
        Ok(instance.into())
    }
}

impl From<Definition> for visualization::Definition {
    fn from(t:Definition) -> Self {
        Self::new(t.signature.clone_ref(),move |scene| t.new_instance(scene))
    }
}


// === Utils ===

fn try_str_field(obj:&JsValue, field:&str) -> Option<String> {
    let field     = js_sys::Reflect::get(obj,&field.into()).ok()?;
    let js_string = field.dyn_ref::<JsString>()?;
    Some(js_string.into())
}

// TODO: convert camel-case names to nice names
fn label(class:&JsValue) -> Result<String,Error> {
    try_str_field(class,LABEL_FIELD).map(Ok).unwrap_or_else(|| {
        let class_name = try_str_field(class,"name").ok_or(Error::InvalidClass(InvalidClass::MissingName))?;
        Ok(class_name)
    })
}



// =============
// === Error ===
// =============

/// Visualization definition or an error occurred during its construction.
pub type FallibleDefinition = Result<Definition,Error>;

/// Error occurred during visualization definition.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum Error {
    InvalidFunction(JsValue),
    InvalidClass(InvalidClass),
}

/// Subset of `Error` related to invalid JavaScript class definition.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum InvalidClass {
    MissingName,
    ConstructorFail(JsValue),
}
