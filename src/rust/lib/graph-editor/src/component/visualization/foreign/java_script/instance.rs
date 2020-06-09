//! This module contains functionality that allows the usage of JavaScript to define visualizations.
//!
//! The `Instance` defines a generic way to wrap JS function calls and allow interaction with
//! JS code and the visualization system.
//!
//! An `Instance` can be created via `Instance::from_object` where the a JS object is provided that
//! fullfills the spec described in `java_script/definition.rs


use crate::prelude::*;

use crate::component::visualization::*;
use crate::component::visualization::java_script::method;
use crate::component::visualization;
use crate::frp;

use core::result;
use ensogl::display::{DomScene, Scene};
use ensogl::display::DomSymbol;
use ensogl::display;
use ensogl::system::web::JsValue;
use ensogl::system::web;
use js_sys;
use std::fmt::Formatter;


// =================
// === Constants ===
// =================

#[allow(missing_docs)]
pub mod constructor {
    pub const ARG: &str = "cls, root";
    pub const BODY : &str = "return new cls(root)";
}



// ==============
// === Errors ===
// ==============

/// Errors that can occur when transforming JS source to a visualization.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum Error {
    /// The provided `JsValue` was expected to be of type `object`, but was not.
    ValueIsNotAnObject { object:JsValue },
    /// The object was expected to have the named property but does not.
    PropertyNotFoundOnObject { object:JsValue, property:String },
    /// An error occurred on the javascript side when calling the class constructor.
    ConstructorError   { js_error:JsValue },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::ValueIsNotAnObject { object }  => {
                f.write_fmt(format_args!("JsValue was expected to be of type `object`, but was \
                                          not: {:?}",object))
            },
            Error::PropertyNotFoundOnObject  { object, property }  => {
                f.write_fmt(format_args!("Object was expected to have property {:?} \
                                          but has not: {:?}",property,object))
            },
            Error::ConstructorError { js_error }  => {
                f.write_fmt(format_args!("Error while constructing object: {:?}",js_error))
            },
        }
    }
}

impl std::error::Error for Error {}

/// Internal helper type to propagate results that can fail due to `JsVisualizationError`s.
pub type Result<T> = result::Result<T, Error>;



// =====================
// === InstanceModel ===
// =====================

/// `JsVisualizationGeneric` allows the use of arbitrary javascript to create visualizations. It
/// takes function definitions as strings and proved those functions with data.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct InstanceModel {
    pub root_node        : DomSymbol,
    pub logger           : Logger,
        on_data_received : Rc<Option<js_sys::Function>>,
        set_size         : Rc<Option<js_sys::Function>>,
        object           : Rc<js_sys::Object>,
}

impl InstanceModel {
    /// Tries to create a InstanceModel from the given visualisation class.
    pub fn from_class(class:&JsValue) -> result::Result<Self, Error> {
        let div       = web::create_div();
        let root_node = DomSymbol::new(&div);
        root_node.dom().set_attribute("id","vis")
            .map_err(|js_error|Error::ConstructorError{js_error})?;

        let js_new   = js_sys::Function::new_with_args(constructor::ARG, constructor::BODY);
        let context  = JsValue::NULL;
        let object   = js_new.call2(&context,&class, &root_node)
            .map_err(|js_error|Error::ConstructorError {js_error})?;
        if !object.is_object() {
            return Err(Error::ValueIsNotAnObject { object } )
        }
        let object:js_sys::Object = object.into();

        let on_data_received = get_method(&object, method::ON_DATA_RECEIVED).ok();
        let on_data_received = Rc::new(on_data_received);
        let set_size         = get_method(&object, method::SET_SIZE).ok();
        let set_size         = Rc::new(set_size);
        let logger           = Logger::new("Instance");
        let object           = Rc::new(object);

        Ok(InstanceModel{object,on_data_received,set_size,root_node,logger}.init())
    }

    fn init(self) -> Self {
        let context   = &self.object;
        if let Ok(init_dom) = get_method(&self.object,method::INIT_DOM) {
            if let Err(err) = init_dom.call1(&context,&self.root_node.dom()) {
                self.logger.warning(
                    || format!("Failed to initialise dom element with error: {:?}",err)
                );
            }
        }
        self
    }

    /// Hooks the root node into the given scene.
    ///
    /// MUST be called to make this visualization visible.
    pub fn set_dom_layer(&self, scene:&DomScene) {
        scene.manage(&self.root_node);
    }

   fn receive_data(&self, data:&Data) -> result::Result<(),DataError> {
       if let Some (on_data_received) = &self.on_data_received.deref() {
           let context   = &self.object;
           let data_json = match data {
               Data::Json {content} => content,
               _ => todo!() // FIXME
           };
           let data_json:&serde_json::Value = data_json.deref();
           let data_js   = match JsValue::from_serde(data_json) {
               Ok(value) => value,
               Err(_)    => return Err(DataError::InvalidDataType),
           };
           if let Err(error) = on_data_received.call1(&context, &data_js) {
               self.logger.warning(
                   || format!("Failed to set data in {:?} with error: {:?}",self,error));
               return Err(DataError::InternalComputationError)
           }
       }
       Ok(())
   }

   fn set_size(&self, size:V2) {
       if let Some(set_size) = &self.set_size.deref() {
           let size          = Vector2::new(size.x,size.y);
           let context       = &self.object;
           let data_json     = JsValue::from_serde(&size).unwrap();
           if let Err(error) = set_size.call1(&context, &data_json) {
               self.logger.warning(
                   || format!("Failed to set size in {:?} with error: {:?}", self, error));
           }
           self.root_node.set_size(size);
       }
   }
}



// ================
// === Instance ===
// ================

/// Sample visualization that renders the given data as text. Useful for debugging and testing.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Instance {
    #[shrinkwrap(main_field)]
    model   : InstanceModel,
    frp     : visualization::instance::Frp,
    network : frp::Network,
}

impl Instance {
    /// Constructor.
    pub fn new(class:&JsValue, scene:&Scene) -> result::Result<Instance, Error>  {
        let network = default();
        let frp     = visualization::instance::Frp::new(&network);
        let model   = InstanceModel::from_class(class)?;
        model.set_dom_layer(&scene.dom.layers.main);

        Ok(Instance{model,frp,network}.init_frp())
    }

    fn init_frp(self) -> Self {
        let network = &self.network;
        let model   = self.model.clone_ref();
        let frp     = self.frp.clone_ref();
        frp::extend! { network
            eval frp.set_size  ((size) model.set_size(*size));
            eval frp.send_data ([frp](data) {
                if let Err(e) = model.receive_data(data) {
                    frp.data_receive_error.emit(Some(e));
                }
             });
        }
        self
    }
}

impl From<Instance> for visualization::Instance {
    fn from(t:Instance) -> Self {
        Self::new(&t,&t.frp,&t.network)
    }
}

impl display::Object for Instance {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.root_node.display_object()
    }
}


// === Utils ===

/// Try to return the method specified by the given name on the given object as a
/// `js_sys::Function`.
fn get_method(object:&js_sys::Object, property:&str) -> Result<js_sys::Function> {
    let method_value  = js_sys::Reflect::get(object,&property.into());
    let method_value  = method_value.map_err(
        |object| Error::PropertyNotFoundOnObject{object,property:property.to_string()})?;

    let method_function:js_sys::Function = method_value.into();
    Ok(method_function)
}
