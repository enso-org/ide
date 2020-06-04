//! The `Registry` provides a mechanism to store `visualization::Class`es for all available visualizations. It
//! provides functionality to register new factories, as well as get suitable factories for
//! a specific data type.
//!
//! Example
//! --------
//! ```no_run
//! use graph_editor::component::visualization::Registry;
//! use graph_editor::component::visualization::EnsoType;
//! use graph_editor::component::visualization::JsSourceClass;
//!
//! // Instantiate a pre-populated registry.
//! let registry = Registry::with_default_visualizations();
//! // Add a new class that creates visualizations defined in JS.
//! registry.register(JsSourceClass::from_js_source_raw(r#"
//!     class BubbleVisualization {
//!         static inputType = "Any"
//!         onDataReceived(root, data) {}
//!         setSize(root, size) {}
//!     }
//!     return BubbleVisualization;
//! "#.into()).unwrap());
//!
//! // Get all factories that can render  visualization for the type `[[Float,Float,Float]]`.
//! let target_type:EnsoType = "[[Float,Float,Float]]".to_string().into();
//! assert!(registry.valid_sources(&target_type).len() > 0);
//! ```

use crate::prelude::*;

use crate::component::visualization;
use crate::component::visualization::*;
use crate::data::EnsoType;

use ensogl::display::scene::Scene;
use crate::component::visualization::example::native::RawText;
use enso_prelude::CloneRef;
use ensogl::data::OptVec;



// ==============================
// === Visualization Registry ===
// ==============================

/// The registry struct. For more information see the module description.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Registry {
    default  : AnyClass,
//    vec      : OptVec<AnyClass>,
    type_map : Rc<RefCell<HashMap<EnsoType,Vec<AnyClass>>>>,
}

impl Registry {
    /// Return an empty `Registry`.
    pub fn new() -> Self {
        let default  = visualization::example::native::RawText::class().into();
        let type_map = Default::default();
        Self {default,type_map} . init()
    }

    fn init(self) -> Self {
        self.add(&self.default);
        self
    }

    /// Return a `Registry` prepopulated with default visualizations.
    pub fn with_default_visualizations() -> Self {
        let registry = Self::new();
        registry.add(visualization::example::native::BubbleChart::class());
        registry.add(visualization::example::native::RawText::class());
        registry.add(visualization::example::js::get_bubble_vis_class());
        registry
    }

    /// Register a new visualization class that's pre-wrapped in an `Rc` with the registry.
    pub fn add(&self, class:impl Into<AnyClass>) {
        let class = class.into();
        let sig   = class.signature();
        self.type_map.borrow_mut().entry(sig.input_type.clone()).or_default().push(class);
    }

    /// Return all `visualization::Class`es that can create a visualization for the given datatype.
    pub fn valid_sources(&self, dtype:&EnsoType) -> Vec<AnyClass>{
        let type_map = self.type_map.borrow();
        type_map.get(dtype).cloned().unwrap_or_else(default)
    }

    /// Return a default visualisation class.
    pub fn default_visualisation(scene:&Scene) -> Visualization {
        let class = visualization::example::native::RawText::class();
        class.instantiate(&scene).expect("Failed to instantiate default visualisation.")
    }
}
