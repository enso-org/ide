//! This module defines the `visualization` struct and related functionality.

use crate::prelude::*;

use crate::frp;
use crate::visualization::*;

use ensogl::display;



// ====================
// === Helper Types ===
// ====================

/// TODO[mm] update this with actual required data for `PreprocessId`
type PreprocessId = String;



// =========================
// === Visualization FRP ===
// =========================

/// Events that are used by the visualization.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct VisualizationFrp {
    pub network              : frp::Network,
    /// Will be emitted if the visualization state changes (e.g., through UI interaction).
    pub on_change            : frp::Source<Option<Data>>,
    /// Will be emitted if the visualization is shown.
    pub on_show              : frp::Source<()>,
    /// Will be emitted if the visualization is hidden.
    pub on_hide              : frp::Source<()>,
    /// Will be emitted if the visualization changes it's preprocessor.
    pub on_preprocess_change : frp::Source<()>,
    /// Will be emitted if the visualization has been provided with invalid data.
    pub on_invalid_data      : frp::Source<()>,
    /// Can be sent to set the data of the visualization.
    pub set_data             : frp::Source<Option<Data>>,
}

impl Default for VisualizationFrp {
    fn default() -> Self {
        frp::new_network! { visualization_events
            def on_change            = source::<Option<Data>> ();
            def on_preprocess_change = source::<()>           ();
            def on_hide              = source::<()>           ();
            def on_show              = source::<()>           ();
            def set_data             = source::<Option<Data>> ();
            def on_invalid_data      = source::<()>           ();
        };
        let network = visualization_events;
        Self {network,on_change,on_preprocess_change,on_hide,on_show,set_data,on_invalid_data}
    }
}

// =====================
// === Visualization ===
// =====================

/// Inner representation of a visualization.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Visualization {
    pub frp          : VisualizationFrp,
    // TODO[mm] consider whether to use a `Box` and be exclusive owner of the DataRenderer.
        renderer     : Rc<dyn DataRenderer>,
        preprocessor : Rc<Option<PreprocessId>>,
        data         : Rc<RefCell<Option<Data>>>,
}

impl display::Object  for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.renderer.display_object()
    }
}

impl Visualization {
    /// Create a new `Visualization` with the given `DataRenderer`.
    pub fn new(renderer:Rc<dyn DataRenderer>) -> Self {
        // FIXME use actual pre-processor functionality.
        let preprocessor = default();
        let frp          = VisualizationFrp::default();
        let data         = default();
        Visualization { frp,renderer,preprocessor,data} . init()
    }

    fn init(self) -> Self {
        let network = &self.frp.network;
        let visualization = &self;
        frp::extend! { network
            def _set_data = visualization.frp.set_data.map(f!((visualization)(data) {
                if let Some(data) = data {
                    if visualization.set_data(data.clone_ref()).is_err() {
                        visualization.frp.on_invalid_data.emit(())
                    }
                }
            }));
        }

        let renderer_frp     = self.renderer.frp();
        let renderer_network = &renderer_frp.network;

        frp::new_bridge_network! { [network,renderer_network]
            def _on_changed = renderer_frp.on_change.map(f!((visualization)(data) {
                visualization.frp.on_change.emit(data)
            }));
        }

        self
    }

    /// Update the visualization with the given data. Returns an error if the data did not match
    /// the visualization.
    fn set_data(&self, data:Data) -> Result<(),DataError> {
        self.renderer.set_data(data)?;
        Ok(())
    }

    /// Set the viewport size of the visualization.
    pub fn set_size(&self, size:Vector2<f32>) {
        self.renderer.set_size(size)
    }
}
