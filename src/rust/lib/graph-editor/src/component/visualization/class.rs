//! This module defines the `Visualization` struct and related functionality.

pub mod js;
pub mod native;

pub use js::*;
pub use native::*;

use crate::prelude::*;

use crate::data::EnsoType;
use crate::data::EnsoCode;
use crate::frp;
use crate::visualization::*;

use ensogl::display::Scene;
use ensogl::display::DomSymbol;
use ensogl::display::Symbol;
use ensogl::display;
use std::error::Error;



// =================
// === Signature ===
// =================

/// Contains general information about a visualization.
#[derive(Clone,Debug,PartialEq)]
#[allow(missing_docs)]
pub struct Signature {
    pub name       : ImString,
    pub input_type : EnsoType,
}

impl Signature {
    pub fn for_any_type(name:impl Into<ImString>) -> Self {
        let name       = name.into();
        let input_type = EnsoType::any();
        Self {name,input_type}
    }
}



// ===========
// === FRP ===
// ===========

/// Events that are used by the visualization.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Frp {
    /// Can be sent to set the data of the visualization.
    pub set_data : frp::Source<Data>,

    /// Will be emitted if the visualization has new data (e.g., through UI interaction).
    /// Data is provides encoded as EnsoCode.
    pub on_change : frp::Stream<EnsoCode>,

    /// Will be emitted if the visualization changes it's preprocessor code.
    pub on_preprocess_change : frp::Stream<EnsoCode>,

    /// Will be emitted if the visualization has been provided with invalid data.
    pub on_invalid_data : frp::Stream<()>,

    // Internal sources that feed the public streams.
    change            : frp::Source<EnsoCode>,
    preprocess_change : frp::Source<EnsoCode>,
    invalid_data      : frp::Source<()>,

}

impl Frp {
    /// Constructor.
    fn new(network: &frp::Network) -> Self {
        frp::extend! { network
            def change            = source();
            def preprocess_change = source();
            def invalid_data      = source();
            def set_data          = source();

            let on_change            = change.clone_ref().into();
            let on_preprocess_change = preprocess_change.clone_ref().into();
            let on_invalid_data      = invalid_data.clone_ref().into();
        };
        Self {on_change,on_preprocess_change,set_data,on_invalid_data,change,preprocess_change
             ,invalid_data}
    }
}



// =====================
// === Visualization ===
// =====================

/// A visualization that can be rendered and interacted with. Provides an frp API.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Visualization {
    pub network : frp::Network,
    pub frp     : Frp,
    renderer    : AnyRenderer
}

impl display::Object for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.renderer.display_object()
    }
}

impl Visualization {
    /// Create a new `Visualization` with the given `DataRenderer`.
    pub fn new<T: DataRenderer + 'static>(renderer:T) -> Self {
        let network  = default();
        let frp      = Frp::new(&network);
        let renderer = AnyRenderer::new(renderer);
        Visualization{frp,renderer,network}.init()
    }

    fn init(self) -> Self {
        let network  = &self.network;
        let renderer = &self.renderer;
        let frp      = &self.frp;
        frp::extend! { network
            def _set_data = self.frp.set_data.map(f!([frp,renderer](data) {
                if renderer.receive_data(data.clone()).is_err() {
                    frp.invalid_data.emit(())
                }
            }));
        }

        let renderer_frp     = self.renderer.frp();
        let renderer_network = &renderer_frp.network;
        frp::new_bridge_network! { [network,renderer_network]
            eval renderer_frp.on_change ((t) frp.change.emit(t));
            eval renderer_frp.on_preprocess_change ((t) frp.preprocess_change.emit(t.clone()));
        }
        self
    }

    /// Set the viewport size of the visualization.
    pub fn set_size(&self, size:V2) {
        self.renderer.set_size(size)
    }
}



// =============
// === Class ===
// =============

/// Specifies a trait that allows the instantiation of `Visualizations`.
///
/// The `Class` provides a way to implement structs that allow the instantiation of specific
/// visualizations, while already providing general information that doesn't require an
/// instantiated visualization, for example, the name or input type of the visualization.
///
/// There are two example implementations: The `JsSourceClass`, which is based on a JS snippet to
/// instantiate `JsRenderer`, and the fairly generic `NativeConstructorClass`, that only requires
/// a function that can create a InstantiationResult. The later can be used as a thin wrapper around
/// the constructor methods of native visualizations.
pub trait Class: Debug {
    /// Provides additional information about the `Class`, for example, which `DataType`s can be
    /// rendered by the instantiated visualization.
    fn signature(&self) -> &Signature;
    /// Create new visualization, that is initialised for the given scene. This can fail if the
    /// `visualization::Class` contains invalid data, for example, JS code that fails to execute,
    /// or if the scene is in an invalid state.
    // TODO consider not providing the scene here, but hooking the the shapes/dom elements into the
    // scene externally.
    fn instantiate(&self, scene:&Scene) -> InstantiationResult;
}


// === Errors ===

/// Indicates that instantiating a `Visualisation` from a `Class` has failed.
#[derive(Debug,Display)]
#[allow(missing_docs)]
pub enum InstantiationError {
    /// Indicates a problem with instantiating a class object.
    InvalidClass         { inner:Box<dyn Error> },
    /// Indicates a problem with instantiating a visualisation from a valid class object.
    InvalidVisualisation { inner:Box<dyn Error> },
}

/// Result of the attempt to instantiate a `Visualization` from a `Class`.
pub type InstantiationResult = Result<Visualization,InstantiationError>;



// ================
// === AnyClass ===
// ================

#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct AnyClass {
    pub class : Rc<dyn Class>
}

impl AnyClass {
    /// Constructor.
    pub fn new<T:Class+'static>(class:T) -> AnyClass {
        let class = Rc::new(class);
        AnyClass {class}
    }
}

impl Class for AnyClass {
    fn signature(&self) -> &Signature {
        self.class.signature()
    }

    fn instantiate(&self, scene:&Scene) -> InstantiationResult {
        self.class.instantiate(scene)
    }
}




