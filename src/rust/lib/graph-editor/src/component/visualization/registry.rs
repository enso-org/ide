//! The `Registry` provides a mechanism to store `visualization::Class`es for all available visualizations. It
//! provides functionality to register new factories, as well as get suitable factories for
//! a specific data type.

use crate::prelude::*;

use crate::component::visualization;
use crate::component::visualization::*;
use crate::data::EnsoType;

use ensogl::display::scene::Scene;
use crate::component::visualization::example::native::RawText;
use enso_prelude::CloneRef;
use ensogl::data::OptVec;



// ================
// === Registry ===
// ================

/// The registry struct. For more information see the module description.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Registry {
    path_map : Rc<RefCell<HashMap<Path,Definition>>>,
    type_map : Rc<RefCell<HashMap<EnsoType,Vec<Definition>>>>,
}

impl Registry {
    /// Constructor.
    pub fn new() -> Self {
        let path_map = Default::default();
        let type_map = Default::default();
        Self {path_map,type_map}
    }

    /// Return a `Registry` pre-populated with default visualizations.
    pub fn with_default_visualizations() -> Self {
        let registry = Self::new();
        registry.add(visualization::example::native::BubbleChart::definition());
        registry.add(visualization::example::native::RawText::definition());
        // FIXME: uncomment and handle error. Use logger to report that the visualization was not registered due to some error.
        // registry.add(visualization::example::java_script::bubble_visualization());
        registry
    }

    /// Register a new visualization class that's pre-wrapped in an `Rc` with the registry.
    pub fn add(&self, class:impl Into<Definition>) {
        let class = class.into();
        let sig   = &class.signature;
        self.type_map.borrow_mut().entry(sig.input_type.clone()).or_default().push(class.clone_ref());
        self.path_map.borrow_mut().entry(sig.path.clone()).insert(class);
    }

    /// Return all `visualization::Class`es that can create a visualization for the given datatype.
    pub fn valid_sources(&self, tp:&EnsoType) -> Vec<Definition>{
        let type_map = self.type_map.borrow();
        type_map.get(tp).cloned().unwrap_or_default()
    }

    /// Return a default visualisation class.
    pub fn default_visualisation(scene:&Scene) -> visualization::Instance {
        let class = visualization::example::native::RawText::definition();
        // FIXME: do not fail the program. If something bad happens, report it and try to rescue.
        class.new_instance(&scene).expect("Failed to instantiate default visualisation.")
    }
}
