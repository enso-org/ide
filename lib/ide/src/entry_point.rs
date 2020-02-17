#![allow(missing_docs)]

use crate::prelude::*;

use crate::transport::web::ConnectingError;
use crate::transport::web::WebSocket;
use crate::view::project::ProjectView;

const FILE_MANAGER_ENDPOINT: &str = "ws://127.0.0.1:30616";

pub struct InitialConfig {
    pub file_manager_endpoint: String
}

impl InitialConfig {
    pub fn new_mock() -> InitialConfig {
        InitialConfig {
            file_manager_endpoint: "ws://127.0.0.1:30616".into()
        }
    }
}

pub async fn connect_to_file_manager(config:InitialConfig) -> Result<WebSocket,ConnectingError> {
    WebSocket::new_connected(config.file_manager_endpoint).await
}

pub async fn setup_project_view(config:InitialConfig) -> Result<ProjectView,failure::Error> {
    let fm_transport = connect_to_file_manager(config).await?;
    println!("Established connection with File Manager at {}", FILE_MANAGER_ENDPOINT);
    let controller = crate::controller::project::Handle::new(fm_transport);
    println!("Project Controller ready");
    let view = ProjectView::new(controller);
    println!("Project View ready");
    Ok(view)
}

pub fn entry_point() {
    std::mem::forget(executor::web::JsExecutor::new_running_global());
    println!("Global executor setup done");

    let config = InitialConfig::new_mock();
    executor::global::spawn(async move {
        println!("Setting up initial project view");
        let error_msg    = "Failed to setup initial project view.";
        let project_view = setup_project_view(config).await.expect(error_msg);
        project_view.forget();
    });
}
