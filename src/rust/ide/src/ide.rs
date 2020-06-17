use crate::prelude::*;

use crate::transport::web::ConnectingError;
use crate::transport::web::WebSocket;
use crate::view::project::ProjectView;
use crate::config::SetupConfig;

use enso_protocol::binary;
use enso_protocol::language_server;
use enso_protocol::project_manager;
use enso_protocol::project_manager::ProjectMetadata;
use enso_protocol::project_manager::ProjectName;
use uuid::Uuid;

pub struct IDE {
}

impl IDE {
    pub fn new() -> Self {
        Self {}
    }

    /// Creates a new running executor with its own event loop. Registers them
/// as a global executor.
///
/// Note: Caller should store or leak this `JsExecutor` so the global
/// spawner won't be dangling.
    pub fn setup_global_executor() -> executor::web::EventLoopExecutor {
        let executor = executor::web::EventLoopExecutor::new_running();
        executor::global::set_spawner(executor.spawner.clone());
        executor
    }

    /// Establishes transport to the file manager server websocket endpoint.
    pub async fn connect_to_project_manager
    (logger:Logger, config:SetupConfig) -> Result<WebSocket,ConnectingError> {
        WebSocket::new_opened(logger,config.project_manager_endpoint).await
    }

    /// Wraps the transport to the project manager server into the client type and registers it within
    /// the global executor.
    pub fn setup_project_manager
    (transport:impl json_rpc::Transport + 'static) -> project_manager::Client {
        let project_manager = project_manager::Client::new(transport);
        executor::global::spawn(project_manager.runner());
        project_manager
    }

    /// Creates a new websocket transport and waits until the connection is properly opened.
    pub async fn new_opened_ws
    (logger:Logger, address:project_manager::IpWithSocket) -> Result<WebSocket,ConnectingError> {
        let endpoint   = format!("ws://{}:{}", address.host, address.port);
        WebSocket::new_opened(logger,endpoint).await
    }

    /// Connect to language server.
    pub async fn open_project
    ( logger          : &Logger
      , json_endpoint   : project_manager::IpWithSocket
      , binary_endpoint : project_manager::IpWithSocket
      , project_name    : impl Str
    ) -> FallibleResult<controller::Project> {
        info!(logger, "Establishing Language Server connections.");
        let client_id     = Uuid::new_v4();
        let json_ws       = Self::new_opened_ws(logger.clone_ref(), json_endpoint).await?;
        let binary_ws     = Self::new_opened_ws(logger.clone_ref(), binary_endpoint).await?;
        let client_json   = language_server::Client::new(json_ws);
        let client_binary = binary::Client::new(logger,binary_ws);
        crate::executor::global::spawn(client_json.runner());
        crate::executor::global::spawn(client_binary.runner());
        let connection_json   = language_server::Connection::new(client_json,client_id).await?;
        let connection_binary = binary::Connection::new(client_binary,client_id).await?;
        Ok(controller::Project::new(logger,connection_json,connection_binary,project_name))
    }

    /// Creates a new project and returns its metadata, so the newly connected project can be opened.
    pub async fn create_project
    (logger:&Logger, project_manager:&impl project_manager::API) -> FallibleResult<ProjectMetadata> {
        let name = constants::DEFAULT_PROJECT_NAME.to_string();
        info!(logger, "Creating a new project named `{name}`.");
        let id = project_manager.create_project(&name).await?.project_id;
        Ok(ProjectMetadata {
            id,
            name        : ProjectName {name},
            last_opened : None,
        })
    }

    /// Open most recent project or create a new project if none exists.
    pub async fn open_most_recent_project_or_create_new
    (logger:&Logger, project_manager:&impl project_manager::API) -> FallibleResult<controller::Project> {
        let projects_to_list = 1;
        let mut response     = project_manager.list_recent_projects(&projects_to_list).await?;
        let project_metadata = if let Some(project) = response.projects.pop() {
            project
        } else {
            Self::create_project(logger,project_manager).await?
        };
        let endpoints = project_manager.open_project(&project_metadata.id).await?;
        Self::open_project(logger,endpoints.language_server_json_address,
                     endpoints.language_server_binary_address,&project_metadata.name.name).await
    }

    /// Sets up the project view, including the controller it uses.
    pub async fn setup_project_view(logger:&Logger,config:SetupConfig)
                                    -> Result<ProjectView,failure::Error> {
        let transport    = Self::connect_to_project_manager(logger.clone_ref(),config).await?;
        let pm           = Self::setup_project_manager(transport);
        let project      = Self::open_most_recent_project_or_create_new(logger,&pm).await?;
        let project_view = ProjectView::new(logger,project).await?;
        Ok(project_view)
    }

    /// This function is the IDE entry point responsible for setting up all views and controllers.
    pub fn run(&self) {
        let logger          = Logger::new("IDE");
        let global_executor = Self::setup_global_executor();
        // We want global executor to live indefinitely.
        std::mem::forget(global_executor);

        let config = SetupConfig::new_local();
        info!(logger, "Starting IDE with the following config: {config:?}");
        executor::global::spawn(async move {
            let error_msg = "Failed to setup initial project view.";
            // TODO [mwu] Once IDE gets some well-defined mechanism of reporting
            //      issues to user, such information should be properly passed
            //      in case of setup failure.
            let project_view = Self::setup_project_view(&logger,config).await.expect(error_msg);
            logger.info("Setup done.");
            project_view.forget();
        });
    }
}
