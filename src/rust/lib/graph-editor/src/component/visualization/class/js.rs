//! This module contains functionality to create a `Class` object from a JS source strings.

use crate::prelude::*;

use crate::component::visualization::JsVisualizationError;
use crate::component::visualization::InstantiationError;
use crate::component::visualization::JsRenderer;
use crate::component::visualization::JsResult;
use crate::component::visualization::InstantiationResult;
use crate::component::visualization::Visualization;
use crate::component::visualization;
use crate::data::*;

use ensogl::display::Scene;
use ensogl::system::web::JsValue;
use js_sys;



// =================
// === Constants ===
// =================

const NAME_FIELD       : &str = "name";
const INPUT_TYPE_FIELD : &str = "inputType";



// =====================
// === JavaScript ===
// =====================

/// Implements the `visualization::Class` for a JS source string.
///
/// Example
/// -------
/// ```no_run
///
/// # use graph_editor::component::visualization::JavaScript;
///
/// JavaScript::from_js_source_raw(r#"
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
pub struct JavaScript {
    js_class  : JavaScriptClassWrapper,
    signature : visualization::Signature,
}

impl JavaScript {
    /// Create a visualization source from piece of JS source code. Signature needs to be inferred.
    pub fn new
    (module:impl Into<LibraryName>, source:impl AsRef<str>) -> Result<Self,JsVisualizationError> {
        let js_class  = JavaScriptClassWrapper::instantiate_class(source)?;
        let signature = js_class.signature(module)?;
        Ok(JavaScript{js_class,signature})
    }
}

impl visualization::Class for JavaScript {
    fn signature(&self) -> &visualization::Signature {
        &self.signature
    }

    fn instantiate(&self, scene:&Scene) -> InstantiationResult {
        let obj = match self.js_class.instantiate() {
            Ok  (obj) => obj,
            Err (err) => return Err(InstantiationError::InvalidClass {inner:err.into()}),
        };
        let renderer = match JsRenderer::from_object(obj) {
            Ok  (obj) => obj,
            Err (err) => return Err(InstantiationError::InvalidClass {inner:err.into()}),
        };
        renderer.set_dom_layer(&scene.dom.layers.main);
        Ok(Visualization::new(renderer))
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
    fn instantiate_class(source:impl AsRef<str>) -> JsResult<JavaScriptClassWrapper> {
        let source      = source.as_ref();
        let context     = JsValue::NULL;
        let constructor = js_sys::Function::new_no_args(source);
        let class       = constructor.call0(&context)?;
        Ok(JavaScriptClassWrapper{class})
    }

    fn signature(&self, module:impl Into<LibraryName>) -> JsResult<visualization::Signature> {
        let input_type = self.input_type().unwrap_or_default();
        let name       = self.name()?;
        let path       = visualization::Path::new(module,name);
        Ok(visualization::Signature::new(path,input_type))
    }

    fn constructor(&self) -> JsResult<js_sys::Function> {
        Ok(js_sys::Reflect::get(&self.prototype()?,&"constructor".into())?.into())
    }

    fn prototype(&self) -> JsResult<JsValue> {
        Ok(js_sys::Reflect::get(&self.class,&"prototype".into())?)
    }

    fn input_type(&self) -> JsResult<EnsoType> {
        let input_type     = js_sys::Reflect::get(&self.class, &INPUT_TYPE_FIELD.into())?;
        let input_type_str = EnsoType::from(input_type.as_string().unwrap()); // FIXME incl check if field exists
        Ok(input_type_str)
    }

    fn name(&self) -> JsResult<visualization::Name> {
        let constructor = self.constructor()?;
        let name        = js_sys::Reflect::get(&constructor,&NAME_FIELD.into())?;
        Ok(name.as_string().unwrap_or_default().into())
    }

    fn instantiate(&self) -> JsResult<JsValue> {
        let fn_wrapper = js_sys::Function::new_with_args("cls", "return new cls()");
        let context    = JsValue::NULL;
        Ok(fn_wrapper.call1(&context, &self.class)?)
    }
}
