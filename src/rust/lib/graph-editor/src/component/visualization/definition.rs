//! `Visualization` struct definition and related functionality.

use crate::prelude::*;

use crate::data::*;
use crate::frp;
use crate::visualization;
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
#[derive(Clone,Debug,Eq,Hash,PartialEq,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Signature {
    #[shrinkwrap(main_field)]
    pub path       : Path,
    pub input_type : EnsoType,
}

impl Signature {
    /// Constructor.
    pub fn new(path:impl Into<Path>, input_type:impl Into<EnsoType>) -> Self {
        let path       = path.into();
        let input_type = input_type.into();
        Self {path,input_type}
    }

    /// Constructor of signature valid for any Enso type.
    pub fn new_for_any_type(path:impl Into<Path>) -> Self {
        let input_type = EnsoType::any();
        Self::new(path,input_type)
    }
}




// =============
// === Definition ===
// =============

/// Trait that allows the instantiation of `Visualizations`.
///
/// The `Definition` provides both a general information about a visualization, so called `Signature`, as
/// well a way to instantiate the visualization.
///
/// There are two generic implementations provided. The `JsSourceClass`, which is based on a JS snippet to
/// instantiate `JsRenderer`, and the fairly generic `NativeConstructorClass`, that only requires
/// a function that can create a InstantiationResult. The later can be used as a thin wrapper around
/// the constructor methods of native visualizations.
pub trait Definition: Debug {

    /// Provides additional information about the `Definition`, for example, which `DataType`s can be
    /// rendered by the instantiated visualization.
    fn signature(&self) -> &Signature;

    /// Create new visualization, that is initialised for the given scene. This can fail if the
    /// `visualization::Definition` contains invalid data, for example, JS code that fails to execute,
    /// or if the scene is in an invalid state.
    // TODO consider not providing the scene here, but hooking the the shapes/dom elements into the
    // scene externally.
    fn new_instance(&self, scene:&Scene) -> InstantiationResult;
}


// === Result ===

/// Result of the attempt to instantiate a `Visualization` from a `Definition`.
pub type InstantiationResult = Result<visualization::Instance,InstantiationError>;


// === Errors ===

/// Indicates that instantiating a `Visualisation` from a `Definition` has failed.
#[derive(Debug,Display)]
#[allow(missing_docs)]
pub enum InstantiationError {
    /// Indicates a problem with instantiating a class object.
    InvalidClass { inner:Box<dyn Error> },

    /// Indicates a problem with instantiating a visualisation from a valid class object.
    InvalidVisualisation { inner:Box<dyn Error> },
}



// ================
// === AnyDefinition ===
// ================

#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct AnyDefinition {
    pub class : Rc<dyn Definition>
}

impl AnyDefinition {
    /// Constructor.
    pub fn new<T:Definition+'static>(class:T) -> AnyDefinition {
        let class = Rc::new(class);
        AnyDefinition {class}
    }
}

impl<T:Definition+'static> From<T> for AnyDefinition {
    fn from(t:T) -> Self {
        Self::new(t)
    }
}
