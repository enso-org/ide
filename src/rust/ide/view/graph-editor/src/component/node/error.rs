//! Contains a struct definition for error information on nodes.
use crate::prelude::*;

use crate::component::node::visualization;



// =============
// === Error ===
// =============

/// Additional error information (beside the error value itself) for some erroneous node.
#[derive(Clone,Debug)]
pub struct Error {
    /// An error message overriding the error visualization data. Should be set in cases when the
    /// visualization won't work (e.g. in case of panics).
    pub message : Option<String>,
    /// Flag indicating that the error is propagated from another node visible on the scene.
    pub propagated : bool,
    // TODO[ao] make use of it.
    pub trace   : Vec<String>,
}

impl Error {
    pub fn visualization_data(&self) -> Option<visualization::Data> {
        let content = serde_json::Value::String(self.message.clone()?).into();
        Some(visualization::Data::Json {content})
    }
}



// // =================
// // === Container ===
// // =================
//
// pub struct Container {
//     error_visualization : builtin_visualization::Error,
// }
//
