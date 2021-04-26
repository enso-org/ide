pub mod cloud;
pub mod local;

use crate::prelude::*;

use mockall::automock;
use enso_protocol::{project_manager, binary, language_server};
use crate::transport::web::WebSocket;
use enso_protocol::project_manager::ProjectName;


// Usually it is a good idea to synchronize this version with the bundled Engine version in
// src/js/lib/project-manager/src/build.ts. See also https://github.com/enso-org/ide/issues/1359
const ENGINE_VERSION_FOR_NEW_PROJECTS : &str = "0.2.10";

pub type ProjectId = Uuid;

#[automock]
pub trait ManagingAPI {
    /// Create new unnamed project.
    fn create_new_unnamed_project<'a>(&'a self) -> BoxFuture<'a, FallibleResult<model::Project>>;

}

#[automock]
pub trait API {
    /// Create model of project opened during application launch
    fn initial_project<'a>(&'a self) -> BoxFuture<'a, FallibleResult<model::Project>>;

    /// Get the API for managing many projects if supported
    fn manage_projects(&self) -> Option<&dyn ManagingAPI>;
}

pub type Manager = Rc<dyn API>;



/// Initializes the json and binary connection to Language Server, and creates a Project Model
pub async fn create_project_model
(logger : &Logger
, project_manager : Option<Rc<dyn project_manager::API>>
, json_endpoint   : String
, binary_endpoint : String
, engine_version  : String
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
    let version           = semver::Version::parse(&engine_version)?;
    let ProjectName(name) = project_name;
    let project           = model::project::Synchronized::from_connections
        (logger,project_manager,connection_json,connection_binary,version,project_id,name).await?;
    Ok(Rc::new(project))
}
