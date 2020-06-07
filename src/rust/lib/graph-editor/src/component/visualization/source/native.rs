use crate::prelude::*;

use super::*;

use crate::component::visualization;
use ensogl::display::Scene;



// ==============
// === Native ===
// ==============

/// Type alias for a function that can create a `Visualization`.
pub trait VisualizationConstructor = Fn(&Scene) -> visualization::InstantiationResult;

/// Constructor that instantiates visualisations from a given `VisualizationConstructor`. Can be
/// used to wrap the constructor of visualizations defined in Rust.
#[derive(Clone,Derivative)]
#[derivative(Debug)]
#[allow(missing_docs)]
pub struct Native {
    #[derivative(Debug="ignore")]
    constructor : Rc<dyn VisualizationConstructor>,
    signature   : visualization::Signature,
}

impl Native {
    /// Create a visualization source from a closure that returns a `Visualization`.
    pub fn new<T>(signature:visualization::Signature, constructor:T) -> Self
    where T: Fn(&Scene) -> visualization::InstantiationResult + 'static {
        let constructor = Rc::new(constructor);
        Native{signature,constructor}
    }
}

impl visualization::Class for Native {
    fn signature(&self) -> &visualization::Signature {
        &self.signature
    }

    fn instantiate(&self, scene:&Scene) -> visualization::InstantiationResult {
        self.constructor.call((scene,))
    }
}
