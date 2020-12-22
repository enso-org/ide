//! This module contains the IDE object implementation.

use crate::prelude::*;

use crate::config;
use crate::transport::web::WebSocket;
use crate::view::View;

use enso_protocol::binary;
use enso_protocol::traits::*;
use enso_protocol::language_server;
use enso_protocol::project_manager;
use enso_protocol::project_manager::ProjectName;
use uuid::Uuid;
use crate::config::BackendService;


// =================
// === Constants ===
// =================

// TODO[ao] We need to set a big timeout on Project Manager to make sure it will have time to
//          download required version of Engine. This should be handled properly when implementing
//          https://github.com/enso-org/ide/issues/1034
const PROJECT_MANAGER_TIMEOUT_SEC:u64 = 2 * 60 * 60;



// ==============
// === Errors ===
// ==============

/// Error raised when project with given name was not found.
#[derive(Clone,Debug,Fail)]
#[fail(display="Project with nae {} was not found.", name)]
pub struct ProjectNotFound {
    name : ProjectName
}



// ===========
// === Ide ===
// ===========

/// The IDE structure containing its configuration and its components instances.
#[derive(Debug)]
pub struct Ide {
    view : View
}




// ======================
// === IdeInitializer ===
// ======================

/// The IDE initializer.
#[derive(Debug)]
pub struct IdeInitializer {
    config : config::Startup,
    logger : Logger
}


impl IdeInitializer {
    pub fn new_local() -> Self {
        Self {
            config : config::Startup::new_local().expect("Failed to load configuration."),
            logger : Logger::new("IdeInitializer"),
        }
    }

    pub fn start_and_forget(self) {
        let executor = setup_global_executor();
        executor::global::spawn(async move {
            info!(self.logger, "Starting IDE with the following config: {self.config:?}");
            // TODO [mwu] Once IDE gets some well-defined mechanism of reporting
            //      issues to user, such information should be properly passed
            //      in case of setup failure.
            let project = self.initialize_project_model().await.expect("Failed to setup project model");
            let view    = View::new(Logger::new("IDE"),project).await.expect("Failed to initialize project view");
            info!(self.logger,"Setup done.");
            let ide = Ide{view};
            std::mem::forget(ide);
        });
        std::mem::forget(executor);
    }

    pub async fn initialize_project_model(&self) -> FallibleResult<model::Project>{
        match &self.config.backend {
            BackendService::ProjectManager { endpoint } => {
                let project_manager = self.setup_project_manager(endpoint).await?;
                let logger          = self.logger.clone_ref();
                let project_name    = self.config.project_name.clone();
                let initializer     = IdeWithProjectManagerInitializer{logger,project_manager,project_name};
                initializer.open_project().await
            }
            BackendService::LanguageServer {json_endpoint,binary_endpoint} => {
                create_project_model(&self.logger,None,json_endpoint.into(),binary_endpoint.into(),default(),self.config.project_name.clone()).await
            }
        }
    }

    /// Wraps the transport to the project manager server into the client type and registers it
    /// within the global executor.
    pub async fn setup_project_manager
    (&self, endpoint:&str) -> FallibleResult<project_manager::Client> {
        let transport           = WebSocket::new_opened(self.logger.clone_ref(),endpoint).await?;
        let mut project_manager = project_manager::Client::new(transport);
        project_manager.set_timeout(std::time::Duration::from_secs(PROJECT_MANAGER_TIMEOUT_SEC));
        executor::global::spawn(project_manager.runner());
        Ok(project_manager)
    }
}

struct IdeWithProjectManagerInitializer {
    logger          : Logger,
    project_manager : project_manager::Client,
    project_name    : ProjectName,
}

impl IdeWithProjectManagerInitializer {
    /// Connect to language server.
    pub async fn open_project(self) -> FallibleResult<model::Project> {
        use project_manager::MissingComponentAction::*;

        let project_id      = self.get_project_or_create_new().await?;
        let endpoints       = self.project_manager.open_project(&project_id,&Install).await?;
        let json_endpoint   = endpoints.language_server_json_address.to_string();
        let binary_endpoint = endpoints.language_server_binary_address.to_string();
        let project_manager:Rc<dyn project_manager::API> = Rc::new(self.project_manager);
        let project_manager = Some(project_manager);
        create_project_model(&self.logger,project_manager,json_endpoint,binary_endpoint,project_id,self.project_name).await
    }

    /// Creates a new project and returns its metadata, so the newly connected project can be
    /// opened.
    pub async fn create_project(&self) -> FallibleResult<Uuid> {
        use project_manager::MissingComponentAction::Install;
        info!(self.logger,"Creating a new project named '{self.project_name}'.");
        let version           = None;
        let ProjectName(name) = &self.project_name;
        let response          = self.project_manager.create_project(name,&version,&Install);
        Ok(response.await?.project_id)
    }

    async fn lookup_project(&self) -> FallibleResult<Uuid> {
        let response     = self.project_manager.list_projects(&None).await?;
        let mut projects = response.projects.iter();
        projects.find(|project_metadata| {
            project_metadata.name == self.project_name
        }).map(|md| md.id).ok_or_else(|| ProjectNotFound{name:self.project_name.clone()}.into())
    }

    /// Returns project with `project_name` or returns a newly created one if it doesn't exist.
    pub async fn get_project_or_create_new(&self) -> FallibleResult<Uuid> {
        let project          = self.lookup_project().await;
        if let Ok(project_id) = project {
            Ok(project_id)
        } else {
            info!(self.logger, "Attempting to create {self.project_name}");
            self.create_project().await
        }
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

async fn create_project_model
( logger : &Logger
, project_manager : Option<Rc<dyn project_manager::API>>
, json_endpoint   : String
, binary_endpoint : String
, project_id      : Uuid
, project_name    : ProjectName
) -> FallibleResult<model::Project> {
    info!(logger, "Establishing Language Server connection.");
    let client_id     = Uuid::new_v4();
    let json_ws       = WebSocket::new_opened(logger,json_endpoint).await?;
    let binary_ws     = WebSocket::new_opened(logger,binary_endpoint).await?;
    let client_json   = language_server::Client::new(json_ws);
    let client_binary = binary::Client::new(logger,binary_ws);
    crate::executor::global::spawn(client_json.runner());
    crate::executor::global::spawn(client_binary.runner());
    let connection_json   = language_server::Connection::new(client_json,client_id).await?;
    let connection_binary = binary::Connection::new(client_binary,client_id).await?;
    let ProjectName(name) = project_name;
    let project           = model::project::Synchronized::from_connections(logger,
        project_manager,connection_json,connection_binary,project_id,name).await?;
    Ok(Rc::new(project))
}