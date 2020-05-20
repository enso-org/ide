//! This module contains functionality to create a `Class` object from a JS source strings.

use crate::prelude::*;

use crate::component::visualization::JsVisualisationError;
use crate::component::visualization::JsRenderer;
use crate::component::visualization::JsResult;
use crate::component::visualization::InstantiationResult;
use crate::component::visualization::Class;
use crate::component::visualization::Visualization;
use crate::component::visualization::ClassAttributes;
use crate::component::visualization::EnsoType;

use ensogl::display::Scene;
use ensogl::system::web::JsValue;
use js_sys;



// ===================================
// === Visualization Class Wrapper ===
// ===================================

/// Internal wrapper for the a JS class that implements our visualization specification. Provides
/// convenience functions for accessing JS methods and attributes.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
struct VisualizationClassWrapper {
    class: JsValue,
}

impl VisualizationClassWrapper {
    fn instantiate_class(source:&str) -> VisualizationClassWrapper {
        let context     = JsValue::NULL;
        let constructor = js_sys::Function::new_no_args(source);
        let class       = constructor.call0(&context).unwrap();
        VisualizationClassWrapper{class}
    }

    fn attributes(&self) -> JsResult<ClassAttributes> {
        let input_types = self.input_types().unwrap_or_default();
        let name        = self.name()?;
        Ok(ClassAttributes{name,input_types})
    }

    fn constructor(&self) -> js_sys::Function {
        js_sys::Reflect::get(&self.prototype(),&"constructor".into()).unwrap().into()
    }

    fn prototype(&self) -> JsValue {
        js_sys::Reflect::get(&self.class,&"prototype".into()).unwrap()
    }

    fn input_types(&self) -> JsResult<Vec<EnsoType>> {
        let fn_get_input_types = js_sys::Reflect::get(&self.prototype(), &"getInputTypes".into())?;
        let fn_get_input_types = js_sys::Function ::from(fn_get_input_types);

        let context                = JsValue::NULL;
        let input_types            = fn_get_input_types.call0(&context)?;
        let input_types            = js_sys::Array::from(&input_types);
        let js_string_to_enso_type = |value:JsValue| {Some(EnsoType::from(value.as_string()?))};
        Ok(input_types.iter().filter_map(js_string_to_enso_type).collect())
    }

    fn name(&self) -> JsResult<String> {
        let name = js_sys::Reflect::get(&self.constructor(),&"name".into())?;
        Ok(name.as_string().unwrap_or_default())
    }

    fn instantiate(&self) -> JsResult<JsValue> {
        let fn_wrapper = js_sys::Function::new_with_args("cls", "return new cls()");
        let context    = JsValue::NULL;
        Ok(fn_wrapper.call1(&context, &self.class)?)
    }
}



// ========================
// === Js Source Class  ===
// ========================

/// Implements the `visualisation::Class` for a JS source string.
///
/// Example
/// -------
/// ```no_run
///
/// # use graph_editor::component::visualization::JsSourceClass;
///
/// JsSourceClass::from_js_source_raw(r#"
///     class Visualisation {
///         onDataReceived(root, data) {}
///         setSize(root, size) {}
///         getInputTypes() {};
///         instantiate() { return new Visualisation(); }
///     }
///     return Visualisations;
/// "#.into()).unwrap();
/// ```
#[derive(CloneRef,Clone,Debug)]
#[allow(missing_docs)]
pub struct JsSourceClass {
    js_class   : Rc<VisualizationClassWrapper>,
    attributes : Rc<ClassAttributes>,
}

impl JsSourceClass {
    /// Create a visualisation source from piece of JS source code. Attributes needs to be inferred.
    pub fn from_js_source_raw(source:&str) -> Result<Self,JsVisualisationError> {
        let js_class   = VisualizationClassWrapper::instantiate_class(&source);
        let attributes = js_class.attributes()?;
        let js_class   = Rc::new(js_class);
        let attributes = Rc::new(attributes);
        Ok(JsSourceClass{js_class,attributes})
    }
}

impl Class for JsSourceClass {
    fn attributes(&self) -> &ClassAttributes {
        &self.attributes
    }

    fn instantiate(&self, scene:&Scene) -> InstantiationResult {
        let obj      = self.js_class.instantiate()?;
        let renderer = JsRenderer::from_object(obj)?;
        renderer.set_dom_layer(&scene.dom.layers.front);
        Ok(Visualization::new(renderer))
    }
}
