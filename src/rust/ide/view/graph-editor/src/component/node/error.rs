//! Contains a struct definition for error information on nodes.
use crate::prelude::*;

use crate::component::node::visualization;

/// Error information to be displayed on a node.
/// Note: this is a dummy implementation that can and should be extended once we have the proper
/// error information from the language server. See #1026 for more information.
#[derive(Clone,Debug)]
pub struct Error {
    /// The error message to show on the node.
    pub message : String,
}

impl Into<visualization::Data> for Error {
    fn into(self) -> visualization::Data {
        let content = serde_json::Value::String(self.message).into();
        visualization::Data::Json {content}
    }
}

impl From<String> for Error {
    fn from(message:String) -> Error {
        Self{message}
    }
}
