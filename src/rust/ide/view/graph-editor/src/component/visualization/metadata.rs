//! Visualization Metadata contains information about the runtime state of the visualization.

use crate::prelude::*;

use crate::data::enso;

/// Description of the visualization state in the IDE.
#[derive(Clone,Debug,Default)]
pub struct Metadata {
    /// An enso lambda that will transform the data into expected format, e.g. `a -> a.json`.
    pub expression: Option<enso::Code>,
}
