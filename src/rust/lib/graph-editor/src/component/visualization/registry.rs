//! The `Registry` provides a mechanism to store all available visualization, as well as select
//! visualizations based in their input type.

use crate::prelude::*;

use crate::component::visualization::EnsoType;
use crate::component::visualization::Visualization;
use crate::component::visualization::renderer::example::js::constructor_sample_js_bubble_chart;
use crate::component::visualization::renderer::example::native::BubbleChart;
use crate::component::visualization::Metadata;
use crate::component::visualization;

use ensogl::display::scene::Scene;



// ==============================
// === Visualization Registry ===
// ==============================

#[derive(Clone,CloneRef,Default,Debug)]
#[allow(missing_docs)]
pub struct Registry {
    entries : Rc<RefCell<Vec<Rc<visualization::Source>>>>
}

impl Registry {

    /// Return an empty `Registry`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Return a `Registry` prepopulated with default visualizations.
    pub fn with_default_visualisations() -> Self {
        let registry = Self::empty();
        // TODO fix types
        registry.register_source(visualization::Source::from_constructor(
            Metadata {
                name       : "Bubble Visualisation (native)".to_string(),
                input_type : "[[float;3]]".to_string().into(),
            },
            Rc::new(|scene:&Scene| Ok(Visualization::new(BubbleChart::new(scene))))
        ));
        registry.register_source(visualization::Source::from_constructor(
            Metadata {
                name       : "Bubble Visualisation (JS)".to_string(),
                input_type : "[[float;3]]".to_string().into(),
            },
            Rc::new(|scene:&Scene| {
                let renderer = constructor_sample_js_bubble_chart();
                renderer.set_dom_layer(&scene.dom.layers.front);
                Ok(Visualization::new(renderer))
            })
        ));

        registry
    }

    /// Register a new visualisation source with the registry.
    pub fn register_source(&self, source: visualization::Source) {
        self.entries.borrow_mut().push(Rc::new(source));
    }

    /// Return all `VisualizationSource`s that can render the given datatype.
    pub fn valid_sources(&self, dtype:&EnsoType) -> Vec<Rc<visualization::Source>>{
        self.entries.borrow().iter().filter(|entry| &entry.metadata().input_type == dtype).cloned().collect()
    }

}
