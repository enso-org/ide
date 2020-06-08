//! This module contains functionality to create a `Class` object from a JS source strings.

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

use super::instance;
use super::instance::Instance;

use ensogl::display::Scene;
use ensogl::system::web::JsValue;
use js_sys;
use js_sys::JsString;



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
        let class      = function.call0(&context).map_err(|e| Error::InvalidFunction(e))?;

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
        let obj     = js_new.call1(&context,&self.class).map_err(|e| InstantiationError::ConstructorError(e))?;
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
    let js_string = JsString::try_from(&field)?;
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

pub type FallibleDefinition = Result<Definition,Error>;

#[derive(Clone,Debug)]
pub enum Error {
    InvalidFunction(JsValue),
    InvalidClass(InvalidClass),
}

#[derive(Clone,Debug)]
pub enum InvalidClass {
    MissingName,
    ConstructorFail(JsValue),
}
