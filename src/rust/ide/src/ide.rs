//! This module contains the IDE object implementation.

use crate::prelude::*;

use crate::transport::web::ConnectingError;
use crate::transport::web::WebSocket;
use crate::view::project::ProjectView;
use crate::config::Startup;

use enso_protocol::binary;
use enso_protocol::language_server;
use enso_protocol::project_manager;
use enso_protocol::project_manager::ProjectMetadata;
use enso_protocol::project_manager::ProjectName;
use uuid::Uuid;

/// The IDE structure containing its configuration and its components instances.
#[derive(Debug)]
pub struct IDE {
    _executor       : executor::web::EventLoopExecutor,
    logger          : Logger,
    config          : Startup,
    project_manager : Option<project_manager::Client>,
    project_view    : Option<ProjectView>
}

impl Default for IDE {
    fn default() -> Self {
        let config          = Startup::new_local();
        let logger          = Logger::new("IDE");
        let _executor       = setup_global_executor();
        let project_view    = default();
        let project_manager = default();
        Self {_executor,logger,config,project_view,project_manager}
    }
}

impl IDE {
    /// Creates a new IDE instance.
    pub fn new() -> Self {
        default()
    }

    /// Establishes transport to the file manager server websocket endpoint.
    pub async fn connect_to_project_manager(&self) -> Result<WebSocket,ConnectingError> {
        WebSocket::new_opened(self.logger.clone_ref(),&self.config.project_manager_endpoint).await
    }

    /// Wraps the transport to the project manager server into the client type and registers it
    /// within the global executor.
    pub fn setup_project_manager
    (transport:impl json_rpc::Transport + 'static) -> project_manager::Client {
        let project_manager = project_manager::Client::new(transport);
        executor::global::spawn(project_manager.runner());
        project_manager
    }

    /// Connect to language server.
    pub async fn open_project
    ( logger          : &Logger
    , json_endpoint   : project_manager::IpWithSocket
    , binary_endpoint : project_manager::IpWithSocket
    , project_name    : impl Str
    ) -> FallibleResult<controller::Project> {
        info!(logger, "Establishing Language Server connection.");
        let client_id     = Uuid::new_v4();
        let json_ws       = new_opened_ws(logger.clone_ref(), json_endpoint).await?;
        let binary_ws     = new_opened_ws(logger.clone_ref(), binary_endpoint).await?;
        let client_json   = language_server::Client::new(json_ws);
        let client_binary = binary::Client::new(logger,binary_ws);
        crate::executor::global::spawn(client_json.runner());
        crate::executor::global::spawn(client_binary.runner());
        let connection_json   = language_server::Connection::new(client_json,client_id).await?;
        let connection_binary = binary::Connection::new(client_binary,client_id).await?;
        Ok(controller::Project::new(logger,connection_json,connection_binary,project_name))
    }

    /// Creates a new project and returns its metadata, so the newly connected project can be
    /// opened.
    pub async fn create_project
    ( logger          : &Logger
    , project_manager : &impl project_manager::API
    , name            : &str
    ) -> FallibleResult<ProjectMetadata> {
        info!(logger, "Creating a new project named '{name}'.");
        let id          = project_manager.create_project(&name.to_string()).await?.project_id;
        let name        = name.to_string();
        let name        = ProjectName{name};
        let last_opened = default();
        Ok(ProjectMetadata{id,name,last_opened})
    }

    /// Open the named project or create a new project if it doesn't exist.
    pub async fn open_project_or_create_new
    ( logger : &Logger
    , project_manager : &impl project_manager::API
    , project_name    : &str
    ) -> FallibleResult<controller::Project> {
        let projects_to_list = constants::MAXIMUM_LISTABLE_PROJECTS;
        let response     = project_manager.list_recent_projects(&projects_to_list).await?;
        let mut projects = response.projects.iter();
        let project      = projects.find(|project_metadata| {
            project_metadata.name.name == *project_name
        });
        let project_metadata = if let Some(project) = project {
            project.clone()
        } else {
            println!("Attempting to create {}", project_name);
            Self::create_project(logger,project_manager,project_name).await?
        };
        let endpoints = project_manager.open_project(&project_metadata.id).await?;
        Self::open_project(logger,endpoints.language_server_json_address,
            endpoints.language_server_binary_address,&project_metadata.name.name).await
    }

    /// Open most recent project or create a new project if none exists.
    pub async fn open_most_recent_project_or_create_new
    ( logger          : &Logger
    , project_manager : &impl project_manager::API) -> FallibleResult<controller::Project> {
        let projects_to_list = 1;
        let mut response     = project_manager.list_recent_projects(&projects_to_list).await?;
        let project_metadata = if let Some(project) = response.projects.pop() {
            project
        } else {
            let project_name = constants::DEFAULT_PROJECT_NAME.to_string();
            Self::create_project(logger,project_manager,&project_name).await?
        };
        let endpoints = project_manager.open_project(&project_metadata.id).await?;
        Self::open_project(logger,endpoints.language_server_json_address,
                     endpoints.language_server_binary_address,&project_metadata.name.name).await
    }

    async fn initialize_project_manager(&mut self) -> FallibleResult<()> {
        let transport        = self.connect_to_project_manager().await?;
        self.project_manager = Some(Self::setup_project_manager(transport));
        Ok(())
    }

    /// Sets up the project view, including the controller it uses.
    pub async fn setup_project_view(&self) -> Result<ProjectView,failure::Error> {
        let logger       = &self.logger;
        let pm           = self.project_manager.as_ref().expect("Couldn't get Project Manager.");
        let arguments    = ensogl::system::web::Arguments::new();
        let project = if let Some(project_name) = arguments.get("project") {
            Self::open_project_or_create_new(logger,pm,project_name).await?
        } else {
            Self::open_most_recent_project_or_create_new(logger,pm).await?
        };
        let project_view = ProjectView::new(logger,project).await?;
        Ok(project_view)
    }

    async fn initialize_project_view(&mut self) -> FallibleResult<()> {
        let project_view = self.setup_project_view().await?;
        self.project_view = Some(project_view);
        Ok(())
    }

    /// This function initializes the project manager, creates the project view and forget IDE
    /// to indefinitely keep it alive.
    pub fn run_and_forget(mut self) {
        info!(self.logger, "Starting IDE with the following config: {self.config:?}");
        executor::global::spawn(async move {
            // TODO [mwu] Once IDE gets some well-defined mechanism of reporting
            //      issues to user, such information should be properly passed
            //      in case of setup failure.
            self.initialize_project_manager().await.expect("Failed to initialize Project Manager.");
            self.initialize_project_view().await.expect("Failed to setup initial project view.");
            self.logger.info("Setup done.");
            std::mem::forget(self);
        });
    }
}

// =============
// === Utils ===
// =============

/// Creates a new running executor with its own event loop. Registers them as a global executor.
pub fn setup_global_executor() -> executor::web::EventLoopExecutor {
    let executor = executor::web::EventLoopExecutor::new_running();
    executor::global::set_spawner(executor.spawner.clone());
    executor
}


/// Creates a new websocket transport and waits until the connection is properly opened.
pub async fn new_opened_ws
(logger:Logger, address:project_manager::IpWithSocket) -> Result<WebSocket,ConnectingError> {
    let endpoint = format!("ws://{}:{}", address.host, address.port);
    WebSocket::new_opened(logger,endpoint).await
}
