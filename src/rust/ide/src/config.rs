//! This module provides IDE configuration structures.

use crate::constants;

/// Configuration data necessary to initialize IDE.
///
/// We will eventually want to load it from a configuration file.
#[derive(Clone,Debug)]
pub struct Startup {
    /// WebSocket endpoint of the project manager service.
    pub project_manager_endpoint:String
}

impl Startup {
    /// Provisional initial configuration that can be used during local deployments.
    pub fn new_local() -> Startup {
        let project_manager_endpoint = constants::PROJECT_MANAGER_ENDPOINT.into();
        Startup{project_manager_endpoint}
    }
}
