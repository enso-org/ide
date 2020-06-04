//! This module defines the `DataRenderer` trait and related functionality.

use crate::prelude::*;
use crate::visualization::*;

use crate::frp;

use ensogl::display;
use crate::data::EnsoCode;


// ===========
// === FRP ===
// ===========

/// FRP API of a `DataRenderer`.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct DataRendererFrp {
    pub network : frp::Network,
    /// This is emitted if the state of the renderer has been changed by UI interaction.
    /// It contains the output data of this visualization if there is some.
    pub on_change : frp::Stream<EnsoCode>,
    /// Will be emitted if the visualization changes it's preprocessor. Transmits the new
    /// preprocessor code.
    pub on_preprocess_change : frp::Stream<EnsoCode>,

    change            : frp::Source<EnsoCode>,
    preprocess_change : frp::Source<EnsoCode>,
}

impl Default for DataRendererFrp {
    fn default() -> Self {
        frp::new_network! { network
            def change            = source();
            def preprocess_change = source();
        };
        let on_change            = change.clone_ref().into();
        let on_preprocess_change = preprocess_change.clone_ref().into();
        Self {network,on_change,on_preprocess_change,change,preprocess_change}
    }
}



// ====================
// === DataRenderer ===
// ====================

/// At the core of the visualization system sits a `DataRenderer`. The DataRenderer is in charge of
/// producing a `display::Object` that will be shown in the scene. It will create FRP events to
/// indicate updates to its output data (e.g., through user interaction).
///
/// A DataRenderer can indicate what kind of data it can use to create a visualization through the
/// `valid_input_types` method. This serves as a hint, it will also reject invalid input in the
/// `set_data` method with a `DataError`. The owner of the `DataRenderer` is in charge of producing
/// UI feedback to indicate a problem with the data.
pub trait DataRenderer: display::Object + Debug {
    /// Receive the data that should be rendered. If the data is valid, it will return the data as
    /// processed by this `DataRenderer`, if the data is of an invalid data type, it violates other
    /// assumptions of this `DataRenderer`, a `DataError` is returned.
    fn receive_data(&self, data:Data) -> Result<(), DataError>;

    /// Set the size of viewport of the visualization. The visualization must not render outside of
    /// this viewport.
    fn set_size(&self, size:V2);

    /// Return a ref to the internal FRP network.
    fn frp(&self) -> &DataRendererFrp;
}



// ===================
// === AnyRenderer ===
// ===================

/// Internal data of Visualization.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct  AnyRenderer {
    renderer : Rc<dyn DataRenderer>,
}

impl AnyRenderer {
    pub fn new<T>(renderer:T) -> Self
    where T : 'static + DataRenderer {
        let renderer = Rc::new(renderer);
        Self {renderer}
    }
}

impl display::Object for AnyRenderer {
    fn display_object(&self) -> &display::object::Instance {
        &self.renderer.display_object()
    }
}
