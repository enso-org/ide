//! This module defines the `Visualization` struct and related functionality.

use crate::prelude::*;

use crate::frp;
use crate::visualization::*;

use ensogl::display;



// ====================
// === Helper Types ===
// ====================

/// Type alias for a string containing enso code.
#[derive(Clone,CloneRef,Debug)]
pub struct  EnsoCode {
    content: Rc<String>
}
/// Type alias for a string representing an enso type.
#[derive(Clone,CloneRef,Debug)]
pub struct  EnsoType {
    content: Rc<String>
}



// =========================
// === Visualization FRP ===
// =========================

/// Events that are used by the visualization.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Frp {
    /// Can be sent to set the data of the visualization.
    pub set_data          : frp::Source<Option<Data>>,
        change            : frp::Source<Option<EnsoCode>>,
        preprocess_change : frp::Source<Option<EnsoCode>>,
        invalid_data      : frp::Source<()>,

    /// Will be emitted if the visualization state changes (e.g., through UI interaction).
    pub on_change            : frp::Stream<Option<EnsoCode>>,
    /// Will be emitted if the visualization changes it's preprocessor.
    pub on_preprocess_change : frp::Stream<Option<EnsoCode>>,
    /// Will be emitted if the visualization has been provided with invalid data.
    pub on_invalid_data      : frp::Stream<()>,
}

impl Frp {
    fn new(network: &frp::Network) -> Self {
        frp::extend! { network
            def change            = source();
            def preprocess_change = source();
            def invalid_data      = source();
            def set_data          = source();

            def on_change            = change.map(|code:&Option<EnsoCode>| code.as_ref().map(|c|c.clone_ref()));
            def on_preprocess_change = preprocess_change.map(|code:&Option<EnsoCode>| code.as_ref().map(|c|c.clone_ref()));
            def on_invalid_data      = invalid_data.map(|_|{});
        };
        Self { on_change,on_preprocess_change,set_data,on_invalid_data,change
              ,preprocess_change,invalid_data}
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
    pub frp      : Frp,
        internal : Rc<Internal>
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
        let frp          = Frp::new(&network);

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
                    if visualization.renderer.receive_data(data.clone_ref()).is_err() {
                        frp.invalid_data.emit(())
                    }
                }
            }));
        }

        let renderer_frp     = self.internal.renderer.frp();
        let renderer_network = &renderer_frp.network;
        frp::new_bridge_network! { [network,renderer_network]
            def _on_changed = renderer_frp.on_change.map(f!((frp)(data) {
                frp.change.emit(data)
            }));
           def _on_changed = renderer_frp.on_preprocess_change.map(f!((frp)(data) {
                frp.preprocess_change.emit(data.as_ref().map(|code|code.clone_ref()))
            }));
        }

        self
    }

    /// Set the viewport size of the visualization.
    pub fn set_size(&self, size:Vector2<f32>) {
        self.internal.renderer.set_size(size)
    }

}
