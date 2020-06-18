//! This module provides constants used throughout the crate.

use crate::constants;

/// Configuration data necessary to initialize IDE.
///
/// Eventually we expect it to be passed to IDE from an external source.
#[derive(Clone,Debug)]
pub struct SetupConfig {
    /// WebSocket endpoint of the project manager service.
    pub project_manager_endpoint:String
}

impl SetupConfig {
    /// Provisional initial configuration that can be used during local deployments.
    pub fn new_local() -> SetupConfig {
        SetupConfig {
            project_manager_endpoint:constants::PROJECT_MANAGER_ENDPOINT.into()
        }
    }
}
