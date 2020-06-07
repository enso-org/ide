//! This module defines the `Renderer` trait and related functionality.

use crate::prelude::*;
use crate::visualization::*;

use crate::frp;

use ensogl::display;
use crate::data::EnsoCode;


// ===========
// === FRP ===
// ===========

/// FRP API of a `Renderer`.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct InstanceFrp {
    /// This is emitted if the state of the renderer has been changed by UI interaction.
    /// It contains the output data of this visualization if there is some.
    pub on_change : frp::Stream<EnsoCode>,
    /// Will be emitted if the visualization changes it's preprocessor. Transmits the new
    /// preprocessor code.
    pub on_preprocess_change : frp::Stream<EnsoCode>,
    pub set_size             : frp::Source<V2>,
    pub send_data            : frp::Source<Data>,

    change            : frp::Source<EnsoCode>,
    preprocess_change : frp::Source<EnsoCode>,
}

impl InstanceFrp {
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def change            = source();
            def preprocess_change = source();
            def set_size          = source();
            def send_data         = source();
        };
        let on_change            = change.clone_ref().into();
        let on_preprocess_change = preprocess_change.clone_ref().into();
        Self {on_change,on_preprocess_change,change,preprocess_change,set_size,send_data}
    }
}



// ====================
// === Renderer ===
// ====================

/// At the core of the visualization system sits a `Renderer`. The Renderer is in charge of
/// producing a `display::Object` that will be shown in the scene. It will create FRP events to
/// indicate updates to its output data (e.g., through user interaction).
///
/// A Renderer can indicate what kind of data it can use to create a visualization through the
/// `valid_input_types` method. This serves as a hint, it will also reject invalid input in the
/// `set_data` method with a `DataError`. The owner of the `Renderer` is in charge of producing
/// UI feedback to indicate a problem with the data.
pub trait Instance: display::Object + Debug {
    /// Return a ref to the internal FRP network.
    fn frp(&self) -> &InstanceFrp;
}




