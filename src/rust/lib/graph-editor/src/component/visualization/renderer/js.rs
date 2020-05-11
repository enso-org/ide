//! This module contains functionality that allows the usage of JavaScript to define visualizations.
//!
//! The `JsRendererGeneric` defines a generic way to wrap JS function calls and allow
//! interaction with JS code and the visualisation system.
//!
//! TODO: the JS visualisations should be based on wrapping a single JS object, instead of
//! multiple functions.
use crate::prelude::*;

use crate::component::visualization::DataRendererFrp;
use crate::component::visualization::Data;
use crate::component::visualization::DataError;
use crate::component::visualization::DataRenderer;

use ensogl::display::DomScene;
use ensogl::display::DomSymbol;
use ensogl::display;
use ensogl::system::web::JsValue;
use ensogl::system::web;
use js_sys;


/// `JsVisualizationGeneric` allows the use of arbitrary javascript to create visualisations. It
/// takes function definitions as strings and proved those functions with data.
///
/// TODO add hooks for status messages form the JS side to the FRP system,
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct JsRendererGeneric {
    set_data    : js_sys::Function,
    set_size    : js_sys::Function,
    /// Root node of this visualisation.
    pub content : DomSymbol,
        frp     : DataRendererFrp,
    pub logger  : Logger,
}

impl JsRendererGeneric {
    /// Constructor.
    ///
    /// `fn_set_data` and `fn_set_size` need to be strings that contain valid JavaScript code. This
    /// code will be executed as the function body of the respective functions. Functions arguments
    /// are available in the body through the `data` variable.
    ///
    /// `fn_set_data` will be called with two arguments: the first argument will be the root node t
    /// that the visualisation should use to build its output, the second argument will be the data
    /// that it should visualise.
    ///
    /// `fn_set_size` will be called with a tuple of floating point values indicating the desired
    /// width and height. This can be used by the visualisation to ensure proper scaling.
    ///
    pub fn new(fn_set_data:&str, fn_set_size:&str) -> Self {
        let set_data = js_sys::Function::new_no_args(fn_set_data);
        let set_size = js_sys::Function::new_no_args(fn_set_size);

        let logger  = Logger::new("JsRendererGeneric");
        let frp     = default();
        let div     = web::create_div();
        let content = DomSymbol::new(&div);
        content.dom().set_attribute("id","vis").unwrap();

        JsRendererGeneric { set_data,set_size,content,frp,logger }
    }

    /// Hooks the root node into the given scene.
    ///
    /// MUST be called to make this visualisation visible.
    // TODO[mm] find a better mechanism to ensure this. Probably through the registry later on.
    pub fn set_dom_layer(&self, scene:&DomScene) {
        scene.manage(&self.content);
    }
}

impl DataRenderer for JsRendererGeneric {

    fn set_data(&self, data: Data) -> Result<(),DataError> {
        let context   = JsValue::NULL;
        let data_json = data.as_json()?;
        let data_js   =  if let Ok(value) = JsValue::from_serde(&data_json) {
            value
        } else {
            return Err(DataError::InvalidDataType)
        };
        if let Err(error) = self.set_data.call2(&context,&self.content.dom(),&data_js,) {
            self.logger.warning(|| format!("Failed to set data in {:?} with error: {:?}", self, error));
            return Err(DataError::InternalComputationError)
        }
        Ok(())
    }

    fn set_size(&self, size: Vector2<f32>) {
        let context       = JsValue::NULL;
        let data_json     = JsValue::from_serde(&size).unwrap();
        if let Err(error) = self.set_size.call2(&context,&self.content.dom(),&data_json) {
            self.logger.warning(|| format!("Failed to set size in {:?} with error: {:?}", self, error));
        }
    }

    fn frp(&self) -> &DataRendererFrp {
        &self.frp
    }
}

impl display::Object for JsRendererGeneric {
    fn display_object(&self) -> &display::object::Instance {
        &self.content.display_object()
    }
}
