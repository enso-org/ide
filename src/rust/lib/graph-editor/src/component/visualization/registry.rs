//! The `Registry` provides a mechanism to store all available visualization, as well as select
//! visualizations based in their input type.

use crate::prelude::*;

use crate::component::visualization::JsRenderer;
use crate::component::visualization::EnsoType;
use crate::component::visualization::Visualization;
use ensogl::display::scene::Scene;
use crate::component::visualization::renderer::example::native::BubbleChart;
use crate::component::visualization::renderer::example::js::constructor_sample_js_bubble_chart;


pub type VisualisationConstructor = dyn Fn(&Scene) -> Result<Visualization, Box<dyn std::error::Error>>;



// ============================
// === Visualization Source ===
// ============================

/// A visualisation source can be used to create visualisations.
#[derive(Derivative)]
#[derivative(Debug)]
pub enum VisualizationSource {
    JS {
        dtype: EnsoType,
        source : CowString
    },
    Native {
        dtype       : EnsoType,
        #[derivative(Debug="ignore")]
        constructor : Box<VisualisationConstructor>
    }
}

impl VisualizationSource {

    pub fn from_constructor(dtype:EnsoType, constructor:Box<VisualisationConstructor>) -> Self {
        VisualizationSource::Native { dtype,constructor }
    }

    pub fn from_js_source(dtype: EnsoType, source:CowString) -> Self {
        VisualizationSource::JS { dtype,source }
    }

    pub fn instantiate(&self, scene:&Scene) -> Result<Visualization, Box<dyn std::error::Error>> {
        match self {
            VisualizationSource::JS     { source, .. }      => {
                let renderer = JsRenderer::from_constructor(&source)?;
                renderer.set_dom_layer(&scene.dom.layers.front);
                Ok(Visualization::new(renderer))
            },
            VisualizationSource::Native { constructor, .. } => { constructor(scene) },
        }
    }

    pub fn dtype(&self) -> &EnsoType {
        match self {
            VisualizationSource::JS     { dtype, .. } => { dtype },
            VisualizationSource::Native { dtype, .. } => { dtype },
        }
     }
}

#[derive(Default,Debug)]
pub struct Registry {
    entries : Vec<Rc<VisualizationSource>>
}

impl Registry {

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_default_visualisations() -> Self {
        let mut registry = Self::default();
        // TODO fix types
        registry.register_source(VisualizationSource::Native {
            dtype: "[[float;3]]".to_string().into(),
            constructor: Box::new(|scene:&Scene| Ok(Visualization::new(BubbleChart::new(scene))))
        });
        registry.register_source(VisualizationSource::Native {
            dtype: "[[float;3]]".to_string().into(),
            constructor: Box::new(|scene:&Scene| {
                let renderer = constructor_sample_js_bubble_chart();
                renderer.set_dom_layer(&scene.dom.layers.front);
                Ok(Visualization::new(renderer))
            })
        });

        registry
    }

    pub fn register_source(&mut self, source:VisualizationSource) {
        self.entries.push(Rc::new(source));
    }

    pub fn valid_sources(&self, dtype:&EnsoType) -> Vec<Rc<VisualizationSource>>{
        self.entries.iter().filter(|entry| entry.dtype() == dtype).cloned().collect()
    }
}
