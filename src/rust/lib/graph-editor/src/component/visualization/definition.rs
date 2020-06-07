//! Visualization definition abstraction.

use crate::prelude::*;

use crate::data::*;
use crate::frp;
use crate::visualization;
use crate::visualization::*;

use ensogl::display::DomSymbol;
use ensogl::display::Scene;
use ensogl::display::Symbol;
use ensogl::display;
use ensogl::system::web::JsValue;
use std::error::Error;



// =================
// === Signature ===
// =================

/// General information about a visualization.
#[derive(Clone,CloneRef,Debug,Eq,Hash,PartialEq,Shrinkwrap)]
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



// ==================
// === Definition ===
// ==================

/// Generic definition of a visualization. Provides information about the visualization `Signature`,
/// and a way to create new instances.
#[derive(Clone,CloneRef,Derivative)]
#[derivative(Debug)]
pub struct Definition {
    pub signature   : Signature,
    #[derivative(Debug="ignore")]
    pub constructor : Rc<dyn Fn(&Scene) -> InstantiationResult>,
}

impl Definition {
    /// Constructor.
    pub fn new<F>(signature:impl Into<Signature>, constructor:F) -> Self
    where F:'static + Fn(&Scene) -> InstantiationResult {
        let signature   = signature.into();
        let constructor = Rc::new(constructor);
        Self {signature,constructor}
    }

    /// Creates a new instance of the visualization.
    pub fn new_instance(&self, scene:&Scene) -> InstantiationResult {
        (self.constructor)(scene)
    }
}


// === Result ===

/// Result of the attempt to instantiate a `Visualization` from a `Definition`.
pub type InstantiationResult = Result<visualization::Instance,InstantiationError>;


// === Errors ===

// TODO: make Display and fix all usages.
/// Indicates that instantiating a `Visualisation` from a `Definition` has failed.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum InstantiationError {
    /// Indicates a problem with instantiating a class object.
    InvalidClass { inner:Box<dyn Error> },

    /// Indicates a problem with instantiating a visualisation from a valid class object.
    InvalidVisualisation { inner:Box<dyn Error> },

    ConstructorError (JsValue),
}
