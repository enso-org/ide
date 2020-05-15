//! Contains functionality related to creating `Visualisations` from different sources..
//!
//! The `Factory` provides a way to implement structs that allow the instantiation of specific
//! visualisations, while already providing general information that doesn't require an
//! instantiated visualization, for example, the name or input type of the visualisation.
//!
//! There are two example implementations: The `JsSourceFactory`, which is based on a JS snippet to
//! instantiate `JsRenderer`, and the fairly generic `NativeConstructorFactory`, that only requires
//! a function that can create a InstantiationResult. The later can be used as a thin wrapper around
//! the constructor methods of native visualizations.
//!
//! Example
//! --------
//! ```
//! use graph_editor::component::visualization::JsSourceFactory;
//! use graph_editor::component::visualization::NativeConstructorFactory;
//! use graph_editor::component::visualization::Metadata;
//! use graph_editor::component::visualization::Visualization;
//! use graph_editor::component::visualization::renderer::example::native::BubbleChart;
//! use ensogl::display::Scene;
//! use std::rc::Rc;
//!
//! // Create a factory from a JS source code snippet.
//! let js_source_factory = JsSourceFactory::from_js_source_raw(r#"
//! class BubbleVisualisation {
//!     onDataReceived(root, data) {}
//!     setSize(root, size) {}
//! }
//! return new BubbleVisualisation();
//! "#.into());
//!
//! // Create a factory that instantiates a `BubbleChart`.
//! let native_bubble_vis_factory = NativeConstructorFactory::new(
//!     Metadata {
//!         name        : "Bubble Visualisation (native)".to_string(),
//!         input_types : vec!["[[float;3]]".to_string().into()],
//!     },
//!     Rc::new(|scene:&Scene| Ok(Visualization::new(BubbleChart::new(scene))))
//! );
//! ```

use crate::prelude::*;
use crate::visualization::*;

use ensogl::display::Scene;
use std::error::Error;



// ============================
// === Visualisation Factory ===
// ============================

/// Result of the attempt to instantiate a `Visualisation` from a `Factory`.
pub type InstantiationResult = Result<Visualization,Box<dyn Error>>;

/// Allows the creation of a specific `DataRenderer`.
pub trait Factory: Debug {
    /// Indicate which `DataType`s can be rendered by this visualization.
    fn metadata(&self) -> &Metadata;
    /// Create new visualisation, that is initialised for the given scene. This can fail if the
    /// `Factory` contains invalid data, for example, JS code that fails to execute, of if the
    /// scene is in an invalid state.
    // TODO consider not allowing failing here and do the checking on instantiation of the factory structs.
    fn instantiate(&self, scene:&Scene) -> InstantiationResult;
}



// =========================
// === JS Source Factory ===
// =========================

#[derive(CloneRef,Clone,Debug)]
#[allow(missing_docs)]
pub struct JsSourceFactory {
    info   : Rc<Metadata>,
    source : Rc<CowString>,
}

impl JsSourceFactory {
    /// Create a visualisation source from piece of JS source code and some metadata.
    pub fn from_js_source(info:Metadata, source:CowString) -> Self {
        let info   = Rc::new(info);
        let source = Rc::new(source);
        JsSourceFactory{ info,source }
    }

    /// Create a visualisation source from piece of JS source code. Metadata needs to be inferred.
    pub fn from_js_source_raw(source:CowString) -> Self {
        // TODO specify a way to provide this information fom raw source files.
        let info  = Rc::new(Metadata {
            name: "Unknown".to_string(),
            input_types: vec![]
        });
        let source = Rc::new(source);
        JsSourceFactory { info,source }
    }
}

impl Factory for JsSourceFactory {

    fn metadata(&self) -> &Metadata {
       &self.info
    }

    fn instantiate(&self, scene:&Scene) -> InstantiationResult {
        let renderer = JsRenderer::from_constructor(&self.source)?;
        renderer.set_dom_layer(&scene.dom.layers.front);
        Ok(Visualization::new(renderer))
    }
}



// ==================================
// === Native Constructor Factory ===
// ==================================

/// Type alias for a function that can create a `Visualisation`.
pub type VisualisationConstructor = dyn Fn(&Scene) -> InstantiationResult;

#[derive(CloneRef,Clone,Derivative)]
#[derivative(Debug)]
#[allow(missing_docs)]
pub struct NativeConstructorFactory {
    info        : Rc<Metadata>,
    #[derivative(Debug="ignore")]
    constructor : Rc<VisualisationConstructor>,
}

impl NativeConstructorFactory {
    /// Create a visualisation source from a closure that returns a `Visualisation`.
    pub fn new(info:Metadata, constructor:Rc<VisualisationConstructor>) -> Self {
        let info = Rc::new(info);
        NativeConstructorFactory { info,constructor }
    }
}


impl Factory for NativeConstructorFactory {
    fn metadata(&self) -> &Metadata {
       &self.info
    }

    fn instantiate(&self, scene:&Scene) -> InstantiationResult {
       self.constructor.call((scene,))
    }
}
