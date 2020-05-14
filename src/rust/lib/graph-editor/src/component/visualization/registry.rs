//! The `Registry` provides a mechanism to store all available visualization, as well as select
//! visualizations based in their input type.

use crate::prelude::*;

use crate::component::visualization::{JsRenderer, Metadata};
use crate::component::visualization::EnsoType;
use crate::component::visualization::Visualization;
use ensogl::display::scene::Scene;
use crate::component::visualization::renderer::example::native::BubbleChart;
use crate::component::visualization::renderer::example::js::constructor_sample_js_bubble_chart;



// ============================
// === Visualization Source ===
// ============================

/// Type alias for a function that can create a `Visualisation`.
pub type VisualisationConstructor = dyn Fn(&Scene) -> Result<Visualization, Box<dyn std::error::Error>>;


/// A visualisation source can be used to create visualisations.
#[derive(Derivative)]
#[derivative(Debug)]
#[allow(missing_docs)]
pub enum VisualizationSource {
    JS {
        info   : Metadata,
        source : CowString
    },
    Native {
        info        : Metadata,
        #[derivative(Debug="ignore")]
        constructor : Box<VisualisationConstructor>
    }
}

impl VisualizationSource {

    /// Create a visualisation source from a closure that returns a `Visualisation`.
    pub fn from_constructor(info:Metadata, constructor:Box<VisualisationConstructor>) -> Self {
        VisualizationSource::Native { info,constructor }
    }

    /// Create a visualisation source from piece of JS source code.
    pub fn from_js_source(info:Metadata, source:CowString) -> Self {
        VisualizationSource::JS { info,source }
    }

    /// Create new visualisation, that is initialised for the given scene. This can fail if the
    /// `VisualizationSource` contains invalid data, for example, JS code that fails to execute.
    pub fn instantiate(&self, scene:&Scene) -> Result<Visualization,Box<dyn std::error::Error>> {
        match self {
            VisualizationSource::JS     { source, .. }     => {
                let renderer = JsRenderer::from_constructor(&source)?;
                renderer.set_dom_layer(&scene.dom.layers.front);
                Ok(Visualization::new(renderer))
            },
            VisualizationSource::Native { constructor, .. } => { constructor(scene) },
        }
    }

    /// Return the metadata of this visualisation source.
    pub fn metadata(&self) -> &Metadata {
        match self {
            VisualizationSource::JS     { info, .. } => { info },
            VisualizationSource::Native { info, .. } => { info },
        }
     }
}



// ==============================
// === Visualization Registry ===
// ==============================

#[derive(Default,Debug)]
#[allow(missing_docs)]
pub struct Registry {
    entries : Vec<Rc<VisualizationSource>>
}

impl Registry {

    /// Return an empty `Registry`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Return a `Registry` prepopulated with default visualizations.
    pub fn with_default_visualisations() -> Self {
        let mut registry = Self::default();
        // TODO fix types
        registry.register_source(VisualizationSource::Native {
            info: Metadata {
                name       : "Bubble Visualisation (native)".to_string(),
                input_type : "[[float;3]]".to_string().into(),
            },
            constructor: Box::new(|scene:&Scene| Ok(Visualization::new(BubbleChart::new(scene))))
        });
        registry.register_source(VisualizationSource::Native {
            info: Metadata {
                name       : "Bubble Visualisation (JS)".to_string(),
                input_type : "[[float;3]]".to_string().into(),
            },
            constructor: Box::new(|scene:&Scene| {
                let renderer = constructor_sample_js_bubble_chart();
                renderer.set_dom_layer(&scene.dom.layers.front);
                Ok(Visualization::new(renderer))
            })
        });

        registry
    }

    /// Register a new visualisation source with the registry.
    pub fn register_source(&mut self, source:VisualizationSource) {
        self.entries.push(Rc::new(source));
    }

    /// Return all `VisualizationSource`s that can render the given datatype.
    pub fn valid_sources(&self, dtype:&EnsoType) -> Vec<Rc<VisualizationSource>>{
        self.entries.iter().filter(|entry| &entry.metadata().input_type == dtype).cloned().collect()
    }
}
