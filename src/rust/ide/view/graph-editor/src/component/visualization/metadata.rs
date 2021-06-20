//! Visualization Metadata contains information about the runtime state of the visualization.

use crate::prelude::*;

use crate::component::visualization;

/// Description of the visualization state, emitted with visualization_enabled event in GraphEditor.
#[derive(Clone,Debug,Default)]
pub struct Metadata {
    /// An Enso lambda, called on the Engine side before sending data to IDE, allowing us to do some
    /// compression or filtering for the best performance. See also _Lazy Visualization_ section
    /// [here](http://dev.enso.org/docs/ide/product/visualizations.html).
    pub preprocessor: visualization::instance::PreprocessorConfiguration,

    /// Path to the definition of this visualization.
    pub visualization_id : Option<visualization::Path>,
}

impl Metadata {
    pub fn new(preprocessor:&visualization::instance::PreprocessorConfiguration,definition:&Option<visualization::Definition>) -> Self {
        Self {
            preprocessor     : preprocessor.clone_ref(),
            visualization_id : definition.as_ref().map(|def| def.signature.path.clone_ref()),
        }
    }
}
