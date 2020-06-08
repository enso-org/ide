//! This module contains functionality that allows the usage of JavaScript to define visualizations.
//!
//! The `Instance` defines a generic way to wrap JS function calls and allow interaction with
//! JS code and the visualization system.
//!
//! There are at the moment three way to generate a `Instance`:
//! 1. `Instance::from_functions` where the bodies of the required functions are provided as
//!    source code.
//! 2. `Instance::from_object` where the a piece of JS code is provided that must evaluate to an
//!     object that has the required methods that will be called at runtime.
//! 3. `Instance::from_constructor`where the body of a constructor function needs to be
//!     provided. The returned object needs to fulfill the same specification as in (2).
//!
//! Right now the only functions supported on the wrapped object are
//!  * `onDataReceived(root, data)`, which receives the html element that the visualization should be
//!     appended on, as well as the data that should be rendered.
//!  * `setSize(root, size)`, which receives the node that the visualization should be appended on,
//!    as well as the intended size.
//!
//! All functions on the class are optional, and methods that are not present, will be handled as
//! no-op by the  `Instance`.
//!
//! TODO: refine spec and add functions as needed, e.g., init, callback hooks or type indicators.

// FIXME: the above docs are not valid anymore. Some functions like `from_functions` were removed.
//        They were removed because there is no point in supporting so many ways of doing that.
//        Lets support only the things that are really used. In fact I would not have such a big
//        problem with the definitions if the code was not just copy-paste between them.

use crate::prelude::*;

use crate::component::visualization;
use crate::component::visualization::Data;
use crate::component::visualization::DataError;

use ensogl::display::DomScene;
use ensogl::display::DomSymbol;
use ensogl::display::Symbol;
use ensogl::display;
use ensogl::system::web::JsValue;
use ensogl::system::web;
use js_sys;
use std::fmt::Formatter;
use crate::frp;



// ==============
// === Errors ===
// ==============

// FIXME: These errors do not tell anything about why they happened! For example, if we have a class
//        in JS which does not define function "onDataReceived", then we will get error
//        "NotAFunction (undefined)". Moreover, if such function is not defined, we should not
//        raise an error! Everything in vis definition should be optional.
//        Moreover, we exactly know hen these errors occur, so please mark them as specific as
//        possible whenever they occur.

// FIXME: the name of the error doesnt make sense. I understand that this is the error which
//        occurs when trying to instantiate visualization? If so, it should be just named `Error`,
//        as we are in `instance` module. After this change other names do not have much sense as
//        well, like `JsResult`.

/// Errors that can occur when transforming JS source to a visualization.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum JsVisualizationError {
    NotAnObject  { inner:JsValue },
    NotAFunction { inner:JsValue },
    /// An unknown error occurred on the JS side. Inspect the content for more information.
    Unknown      { inner:JsValue }
}

impl Display for JsVisualizationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO find a nice way to circumvent the fact that `JsValue` does not implement `Display`.
        match self {
            JsVisualizationError::NotAnObject { inner }  => {
                f.write_fmt(format_args!("Not an object: {:?}",inner))
            },
            JsVisualizationError::NotAFunction { inner } => {
                f.write_fmt(format_args!("Not a function: {:?}",inner))
            },
            JsVisualizationError::Unknown { inner }      => {
                f.write_fmt(format_args!("Unknown: {:?}",inner))
            },
        }
    }
}

impl std::error::Error for JsVisualizationError {}

impl From<JsValue> for JsVisualizationError {
    fn from(value:JsValue) -> Self {
        // TODO add differentiation if we encounter specific errors and return new variants.
        JsVisualizationError::Unknown {inner:value}
    }
}

/// Internal helper type to propagate results that can fail due to `JsVisualizationError`s.
pub type JsResult<T> = Result<T, JsVisualizationError>;



// ================
// === Instance ===
// ================

/// `JsVisualizationGeneric` allows the use of arbitrary javascript to create visualizations. It
/// takes function definitions as strings and proved those functions with data.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct Instance {
    pub root_node        : DomSymbol,
    pub logger           : Logger,
        on_data_received : js_sys::Function,
        set_size         : js_sys::Function,
        frp              : visualization::instance::Frp,
}

impl Instance {
    /// Internal helper that tries to convert a JS object into a `Instance`.
    fn from_object_js(object:js_sys::Object) -> Result<Instance,JsVisualizationError> {
        let on_data_received = js_sys::Reflect::get(&object,&"onDataReceived".into())?;
        let set_size         = js_sys::Reflect::get(&object,&"setSize".into())?;
        if !on_data_received.is_function() {
            return Err(JsVisualizationError::NotAFunction { inner:on_data_received })
        }
        if !set_size.is_function() {
            return Err(JsVisualizationError::NotAFunction { inner:set_size })
        }
        let on_data_received:js_sys::Function = on_data_received.into();
        let set_size:js_sys::Function = set_size.into();

        let logger    = Logger::new("Instance");
        let frp       = visualization::instance::Frp::new();
        let div       = web::create_div();
        let root_node = DomSymbol::new(&div);
        root_node.dom().set_attribute("id","vis")?;

        Ok(Instance { on_data_received,set_size,root_node,frp,logger })
    }

    /// Constructor from a source that evaluates to an object with specific methods.
    pub fn from_object_source(source: &str) -> Result<Instance,JsVisualizationError> {
        let object = js_sys::eval(source)?;
        if !object.is_object() {
            return Err(JsVisualizationError::NotAnObject { inner:object } )
        }
        Self::from_object_js(object.into())
    }

    pub fn from_object(object:JsValue) -> Result<Instance,JsVisualizationError> {
        if !object.is_object() {
            return Err(JsVisualizationError::NotAnObject { inner:object } )
        }
        Self::from_object_js(object.into())
    }

    /// Constructor from function body that returns a object with specific functions.
    pub fn from_constructor(source:&str) -> Result<Instance,JsVisualizationError> {
        let context     = JsValue::NULL;
        let constructor = js_sys::Function::new_no_args(source);
        let object      = constructor.call0(&context)?;
        if !object.is_object() {
            return Err(JsVisualizationError::NotAnObject { inner:object } )
        }
        Self::from_object_js(object.into())
    }

    /// Hooks the root node into the given scene.
    ///
    /// MUST be called to make this visualization visible.
    pub fn set_dom_layer(&self, scene:&DomScene) {
        scene.manage(&self.root_node);
    }
}

// FIXME: Move to FRP

//impl visualization::InstanceX for Instance {
//
////    fn receive_data(&self, data:Data) -> Result<(),DataError> {
////        let context   = JsValue::NULL;
////        let data_json = match data {
////            Data::Json {content} => content,
////            _ => todo!() // FIXME
////        };
////        let data_js   = match JsValue::from_serde(&data_json) {
////            Ok(value) => value,
////            Err(_)    => return Err(DataError::InvalidDataType),
////        };
////        if let Err(error) = self.on_data_received.call2(&context, &self.root_node.dom(), &data_js) {
////            self.logger.warning(
////                || format!("Failed to set data in {:?} with error: {:?}",self,error));
////            return Err(DataError::InternalComputationError)
////        }
////        Ok(())
////    }
//
////    fn set_size(&self, size:V2) {
////        let size          = Vector2::new(size.x,size.y);
////        let context       = JsValue::NULL;
////        let data_json     = JsValue::from_serde(&size).unwrap();
////        if let Err(error) = self.set_size.call2(&context, &self.root_node.dom(), &data_json) {
////            self.logger.warning(
////                || format!("Failed to set size in {:?} with error: {:?}", self, error));
////        }
////        self.root_node.set_size(size);
////    }
//
//    fn frp(&self) -> &visualization::instance::Frp {
//        &self.frp
//    }
//}

impl From<Instance> for visualization::Instance {
    fn from(t:Instance) -> Self {
        Self::new(&t,&t.frp)
    }
}

impl display::Object for Instance {
    fn display_object(&self) -> &display::object::Instance {
        &self.root_node.display_object()
    }
}
