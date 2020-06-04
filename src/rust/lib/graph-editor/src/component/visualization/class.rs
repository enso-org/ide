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



// =====================
// === Visualization ===
// =====================

/// Internal data of Visualization.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct  Visualization {
    renderer : Rc<dyn Renderer>,
}

impl Deref for Visualization {
    type Target = RendererFrp;
    fn deref(&self) -> &Self::Target {
        self.renderer.frp()
    }
}

impl Visualization {
    pub fn new<T>(renderer:T) -> Self
        where T : 'static + Renderer {
        let renderer = Rc::new(renderer);
        Self {renderer}
    }
}

impl display::Object for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.renderer.display_object()
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




