//! This module contains functionality to create a `Class` object from a JS source strings.

use crate::prelude::*;

use crate::component::visualization::InstantiationResult;
use crate::component::visualization;
use crate::data::*;

use super::instance;
use super::instance::Instance;

use ensogl::display::Scene;
use ensogl::system::web::JsValue;
use js_sys;



// =================
// === Constants ===
// =================

const NAME_FIELD       : &str = "name";
const INPUT_TYPE_FIELD : &str = "inputType";



// =====================
// === Definition ===
// =====================

/// Implements the `visualization::Class` for a JS source string.
///
/// Example
/// -------
/// ```no_run
///
/// # use graph_editor::component::visualization::Definition;
///
/// Definition::from_js_source_raw(r#"
///     class Visualization {
///         static inputType = "Any"
///         onDataReceived(root, data) {}
///         setSize(root, size) {}
///     }
///     return Visualizations;
/// "#.into()).unwrap();
/// ```
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct Definition {
    js_class  : JavaScriptClassWrapper,
    signature : visualization::Signature,
}

impl Definition {
    /// Create a visualization source from piece of JS source code. Signature needs to be inferred.
    pub fn new
    (module:impl Into<LibraryName>, source:impl AsRef<str>) -> Result<Self,instance::JsVisualizationError> {
        let js_class  = JavaScriptClassWrapper::instantiate_class(source)?;
        let signature = js_class.signature(module)?;
        Ok(Definition{js_class,signature})
    }
}

impl visualization::Definition for Definition {
    fn signature(&self) -> &visualization::Signature {
        &self.signature
    }

    fn new_instance(&self, scene:&Scene) -> InstantiationResult {
        let obj = match self.js_class.new_instance() {
            Ok  (obj) => obj,
            Err (err) => return Err(visualization::InstantiationError::InvalidClass {inner:err.into()}),
        };
        let renderer = match Instance::from_object(obj) {
            Ok  (obj) => obj,
            Err (err) => return Err(visualization::InstantiationError::InvalidClass {inner:err.into()}),
        };
        renderer.set_dom_layer(&scene.dom.layers.main);
        Ok(renderer.into())
    }
}



// ======================
// === JavaScriptClassWrapper ===
// ======================

/// Internal wrapper for the a JS class that implements the visualization specification. Provides
/// convenience functions for accessing JS methods and signature.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
struct JavaScriptClassWrapper {
    class : JsValue,
}

impl JavaScriptClassWrapper {
    fn instantiate_class(source:impl AsRef<str>) -> instance::JsResult<JavaScriptClassWrapper> {
        let source      = source.as_ref();
        let context     = JsValue::NULL;
        let constructor = js_sys::Function::new_no_args(source);
        let class       = constructor.call0(&context)?;
        Ok(JavaScriptClassWrapper{class})
    }

    fn signature(&self, module:impl Into<LibraryName>) -> instance::JsResult<visualization::Signature> {
        let input_type = self.input_type().unwrap_or_default();
        let name       = self.name()?;
        let path       = visualization::Path::new(module,name);
        Ok(visualization::Signature::new(path,input_type))
    }

    fn constructor(&self) -> instance::JsResult<js_sys::Function> {
        Ok(js_sys::Reflect::get(&self.prototype()?,&"constructor".into())?.into())
    }

    fn prototype(&self) -> instance::JsResult<JsValue> {
        Ok(js_sys::Reflect::get(&self.class,&"prototype".into())?)
    }

    fn input_type(&self) -> instance::JsResult<EnsoType> {
        let input_type     = js_sys::Reflect::get(&self.class, &INPUT_TYPE_FIELD.into())?;
        let input_type_str = EnsoType::from(input_type.as_string().unwrap()); // FIXME incl check if field exists
        Ok(input_type_str)
    }

    fn name(&self) -> instance::JsResult<visualization::Name> {
        let constructor = self.constructor()?;
        let name        = js_sys::Reflect::get(&constructor,&NAME_FIELD.into())?;
        Ok(name.as_string().unwrap_or_default().into())
    }

    fn new_instance(&self) -> instance::JsResult<JsValue> {
        let fn_wrapper = js_sys::Function::new_with_args("cls", "return new cls()");
        let context    = JsValue::NULL;
        Ok(fn_wrapper.call1(&context, &self.class)?)
    }
}
