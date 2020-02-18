#![allow(missing_docs)]

use crate::prelude::*;

use crate::transport::web::ConnectingError;
use crate::transport::web::WebSocket;
use crate::view::project::ProjectView;



// =====================
// === InitialConfig ===
// =====================

/// Endpoint used by default by a locally run mock file manager server.
const MOCK_FILE_MANAGER_ENDPOINT: &str = "ws://127.0.0.1:30616";

/// Configuration data necessary to initialize IDE.
///
/// Eventually we expect it to be passed to IDE from an external source.
#[derive(Clone,Debug)]
pub struct InitialConfig {
    /// WebSocket endpoint of the file manager service.
    pub file_manager_endpoint: String
}

impl InitialConfig {
    /// Provisional initial configuration that can be used during mock
    /// deployments (manually run mock file manager server).
    pub fn new_mock() -> InitialConfig {
        InitialConfig {
            file_manager_endpoint: MOCK_FILE_MANAGER_ENDPOINT.into()
        }
    }
}

/// Establishes connection with file manager server.
pub async fn connect_to_file_manager(config:InitialConfig) -> Result<WebSocket,ConnectingError> {
    WebSocket::new_opened(config.file_manager_endpoint).await
}

/// Sets up the project view, including the controller it uses.
pub async fn setup_project_view(config:InitialConfig) -> Result<ProjectView,failure::Error> {
    let fm_transport = connect_to_file_manager(config).await?;
    let controller   = crate::controller::project::Handle::new(fm_transport);
    let project_view = ProjectView::new(controller);
    Ok(project_view)
}



// ===================
// === Entry Point ===
// ===================

pub fn entry_point() {
    std::mem::forget(executor::web::JsExecutor::new_running_global());
    let config = InitialConfig::new_mock();
    executor::global::spawn(async move {
        let error_msg    = "Failed to setup initial project view.";
        let project_view = setup_project_view(config).await.expect(error_msg);
        project_view.forget();
    });
}
