//! This module defines the `Visualisation` struct and related functionality.

use crate::prelude::*;

use crate::frp;
use crate::visualization::*;

use ensogl::display;
use fmt;



// ====================
// === Helper Types ===
// ====================

/// TODO[mm] update this with actual required data for `PreprocessId`
type PreprocessId = String;
type PreprocessorCallback = Rc<dyn Fn(Rc<dyn Fn(PreprocessId)>)>;



// =========================
// === Visualization FRP ===
// =========================

/// Events that are emitted by the visualisation.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct VisualisationFrp {
    pub network           : frp::Network,
    /// Will be emitted if the visualisation state changes (e.g., through UI interaction).
    pub on_change            : frp::Source<Option<Data>>,
    pub on_show              : frp::Source<()>,
    pub on_hide              : frp::Source<()>,
    pub on_preprocess_change : frp::Source<()>,
}

impl Default for VisualisationFrp {
    fn default() -> Self {
        frp::new_network! { visualization_events
            def on_change            = source::<Option<Data>> ();
            def on_preprocess_change = source::<()>           ();
            def on_hide              = source::<()>           ();
            def on_show              = source::<()>           ();
        };
        let network = visualization_events;
        Self {network,on_change,on_preprocess_change,on_hide,on_show}
    }
}

// =====================
// === Visualization ===
// =====================

/// Inner representation of a visualisation.
#[derive(Clone)]
#[allow(missing_docs)]
pub struct Visualization {
    pub frp                  : VisualisationFrp,
    // TODO[mm] consider whether to use a `Box` and be exclusive owner of the DataRenderer.
        renderer             : Rc<dyn DataRenderer>,
        preprocessor         : Option<PreprocessId>,
        on_preprocess_change : Option<PreprocessorCallback>,
}

impl Debug for Visualization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO[mm] extend to provide actually useful information.
        f.write_str("<Visualisation>")
    }
}

impl display::Object  for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.renderer.display_object()
    }
}

impl Visualization {

    /// Create a new `Visualization` with the given `DataRenderer`.
    pub fn new(renderer:Rc<dyn DataRenderer>) -> Self {
        let preprocessor         = None;
        let on_preprocess_change = None;
        let frp                  = VisualisationFrp::default();
        Visualization { frp,renderer,preprocessor,on_preprocess_change}
            .init()
    }

    fn init(self) -> Self {
        // TODO hook up renderer frp
        self
    }

    /// Update the visualisation with the given data. Returns an error if the data did not match
    /// the visualization.
    pub fn set_data(&self, data:Data) -> Result<(),DataError> {
        let output_data = self.renderer.set_data(data)?;
        self.frp.on_change.emit(&Some(output_data));
        Ok(())
    }

    /// Set the viewport size of the visualisation.
    pub fn set_size(&self, size: Vector2<f32>) {
        self.renderer.set_size(size)
    }
}

