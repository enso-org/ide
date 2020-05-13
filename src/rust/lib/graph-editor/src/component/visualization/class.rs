//! This module defines the `Visualization` struct and related functionality.

use crate::prelude::*;

use crate::frp;
use crate::visualization::*;

use ensogl::display;



// ====================
// === Helper Types ===
// ====================

/// Type alias for a string containing enso code.
pub type EnsoCode = String;
/// Type alias for a string representing an enso type.
pub type EnsoType = String;



// =========================
// === Visualization FRP ===
// =========================

/// Events that are used by the visualization.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Frp {
    /// Will be emitted if the visualization state changes (e.g., through UI interaction).
    pub on_change            : frp::Source<Option<EnsoCode>>,
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

impl Frp {
    fn new(network: &frp::Network) -> Self {
        frp::extend! { network
            def on_change            = source();
            def on_preprocess_change = source();
            def on_hide              = source();
            def on_show              = source();
            def set_data             = source();
            def on_invalid_data      = source();
        };
        Self {on_change,on_preprocess_change,on_hide,on_show,set_data,on_invalid_data}
    }
}



// ===============================
// === Visualization Internals ===
// ===============================

/// Internal data of Visualization.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Internal {
    pub renderer     : Rc<dyn DataRenderer>,
    pub preprocessor : Rc<RefCell<Option<EnsoCode>>>,
}

impl display::Object for Internal {
    fn display_object(&self) -> &display::object::Instance {
        &self.renderer.display_object()
    }
}

/// A visualization that can be rendered and interacted with. Provides an frp API.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Visualization {
    pub network  : frp::Network,
    pub frp      : Rc<Frp>,
    pub internal : Rc<Internal>
}

impl display::Object for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.internal.display_object()
    }
}

impl Visualization {
    /// Create a new `Visualization` with the given `DataRenderer`.
    pub fn new<T: DataRenderer + 'static>(renderer:Rc<T>) -> Self {
        let preprocessor = default();
        let network      = default();
        let frp          = Rc::new(Frp::new(&network));

        let internal = Rc::new(Internal{preprocessor,renderer});
        Visualization{frp,internal,network}.init()
    }

    fn init(self) -> Self {
        let network       = &self.network;
        let visualization = &self.internal;
        let frp           = &self.frp;
        frp::extend! { network
            def _set_data = self.frp.set_data.map(f!((frp,visualization)(data) {
                if let Some(data) = data {
                    if visualization.renderer.set_data(data.clone_ref()).is_err() {
                        frp.on_invalid_data.emit(())
                    }
                }
            }));
        }

        let renderer_frp     = self.internal.renderer.frp();
        let renderer_network = &renderer_frp.network;
        frp::new_bridge_network! { [network,renderer_network]
            def _on_changed = renderer_frp.on_change.map(f!((frp)(data) {
                frp.on_change.emit(data)
            }));
        }

        self
    }

    /// Set the viewport size of the visualization.
    pub fn set_size(&self, size:Vector2<f32>) {
        self.internal.renderer.set_size(size)
    }

}
