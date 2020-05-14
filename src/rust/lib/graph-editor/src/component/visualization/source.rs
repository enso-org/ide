//! Contains functionality related to creating `Visualisations` from `Source`s.
use crate::prelude::*;

use crate::component::visualization::Visualization;
use crate::component::visualization::JsRenderer;
use crate::component::visualization::Metadata;

use ensogl::display::scene::Scene;

// ============================
// === Visualization Source ===
// ============================

/// Type alias for a function that can create a `Visualisation`.
pub type VisualisationConstructor = dyn Fn(&Scene) -> Result<Visualization, Box<dyn std::error::Error>>;


/// A visualisation source can be used to create visualisations.
#[derive(CloneRef,Clone,Derivative)]
#[derivative(Debug)]
#[allow(missing_docs)]
pub enum Source {
    JS {
        info   : Rc<Metadata>,
        source : Rc<CowString>,
    },
    Native {
        info        : Rc<Metadata>,
        #[derivative(Debug="ignore")]
        constructor : Rc<VisualisationConstructor>
    }
}

impl Source {

    /// Create a visualisation source from a closure that returns a `Visualisation`.
    pub fn from_constructor(info:Metadata, constructor:Rc<VisualisationConstructor>) -> Self {
        let info = Rc::new(info);
        Source::Native { info,constructor }
    }

    /// Create a visualisation source from piece of JS source code and some metadata.
    pub fn from_js_source(info:Metadata, source:CowString) -> Self {
        let info   = Rc::new(info);
        let source = Rc::new(source);
        Source::JS { info,source }
    }

    /// Create a visualisation source from piece of JS source code. Metadata needs to be inferred.
    pub fn from_js_source_raw(source:CowString) -> Self {
        // TODO specify a way to provide this information fom raw source files.
        let info   = Rc::new(Metadata{
            name: "Unknown".to_string(),
            input_types: vec![]
        });
        let source = Rc::new(source);
        Source::JS { info,source }
    }

    /// Create new visualisation, that is initialised for the given scene. This can fail if the
    /// `VisualizationSource` contains invalid data, for example, JS code that fails to execute.
    pub fn instantiate(&self, scene:&Scene) -> Result<Visualization,Box<dyn std::error::Error>> {
        match self {
            Source::JS     { source, .. }     => {
                let renderer = JsRenderer::from_constructor(&source)?;
                renderer.set_dom_layer(&scene.dom.layers.front);
                Ok(Visualization::new(renderer))
            },
            Source::Native { constructor, .. } => { constructor(scene) },
        }
    }

    /// Return the metadata of this visualisation source.
    pub fn metadata(&self) -> &Metadata {
        match self {
            Source::JS     { info, .. } => { &info },
            Source::Native { info, .. } => { &info },
        }
    }
}

