//! This module defines the `DataRenderer` trait and related functionality.

use crate::prelude::*;
use crate::visualization::*;

use crate::frp;

use ensogl::display;



// ===========
// === FRP ===
// ===========

/// FRP api of a `DataRenderer`.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct DataRendererFrp {
    pub network   : frp::Network,
    pub on_change : frp::Source<Option<Data>>,
    pub on_show   : frp::Source<()>,
    pub on_hide   : frp::Source<()>,
}

impl Default for DataRendererFrp {
    fn default() -> Self {
        frp::new_network! { renderer_events
            def on_change = source::<Option<Data>> ();
            def on_show   = source::<()>           ();
            def on_hide   = source::<()>           ();
        };
        let network = renderer_events;
        Self {network,on_change,on_show,on_hide}
    }
}



// ====================
// === DataRenderer ===
// ====================

/// At the core of the visualization system sits a `DataRenderer`. The DataRenderer is in charge of
/// producing a `display::Object` that will be shown in the scene. It will create FRP events to
/// indicate updates to its output data (e.g., through user interaction).
///
/// A DataRenderer can indicate what kind of data it can use to create a visualisation through the
/// `valid_input_types` method. This serves as a hint, it will also reject invalid input in the
/// `set_data` method with a `DataError`. The owner of the `DataRenderer` is in charge of producing
/// UI feedback to indicate a problem with the data.
pub trait DataRenderer: display::Object {
    /// Indicate which `DataType`s can be rendered by this visualization.
    fn valid_input_types(&self) -> Vec<DataType>;
    /// Set the data that should be rendered. If the data is valid, it will return the data as
    /// processed by this `DataRenderer`, if the data is of an invalid data type, ir violates other
    /// assumptions of this `DataRenderer`, a `DataError` is returned.
    /// TODO[mm] reconsider returning the data here. Maybe just have an FRP event.
    fn set_data(&self, data:Data) -> Result<Data, DataError>;
    /// Set the size of viewport of the visualization. The visualisation must not render outside of
    /// this viewport. TODO[mm] define and ensure consistent origin of viewport.
    fn set_size(&self, size:Vector2<f32>);

    /// Return a ref to the internal FRP network. This replaces a potential callback mechanism.
    ///
    /// Note: the presence of this functions imposes the requirement that a `DataRendererFrp` is
    /// owned by whoever implements this trait.
    fn frp(&self) -> &DataRendererFrp;
}
