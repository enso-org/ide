//! This module provides IDE configuration structures.
use crate::prelude::*;

use crate::constants;

use enso_protocol::project_manager::ProjectName;
use ensogl::system::web;



// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[derive(Clone,Debug,Fail)]
#[fail(display="Missing program option: {}.",name)]
pub struct MissingOption {name:&'static str}


// ==================
// === Connection ===
// ==================

#[derive(Clone,Debug)]
pub enum BackendService {
    ProjectManager {endpoint:String},
    LanguageServer {
        json_endpoint: String,
        binary_endpoint: String,
    }
}

impl BackendService {
    fn from_web_arguments(arguments:&web::Arguments) -> FallibleResult<Self> {
        let pm_endpoint      = arguments.get("project_manager").cloned();
        let ls_json_endpoint = arguments.get("language_server_json").cloned();
        let ls_bin_endpoint  = arguments.get("language_server_binary").cloned();
        match (pm_endpoint,ls_json_endpoint,ls_bin_endpoint) {
            (None,None,None) =>
                Ok(Self::ProjectManager {endpoint:constants::PROJECT_MANAGER_ENDPOINT.into()}),
            (Some(endpoint),_,_) =>
                Ok(Self::ProjectManager {endpoint}),
            (None,Some(json_endpoint),Some(binary_endpoint)) =>
                Ok(Self::LanguageServer {json_endpoint,binary_endpoint}),
            (None,Some(_),None   ) => Err(MissingOption{name:"language_server_binary"}.into()),
            (None,None   ,Some(_)) => Err(MissingOption{name:"language_server_json"  }.into())
        }
    }
}

/// Configuration data necessary to initialize IDE.
///
/// We will eventually want to load it from a configuration file.
#[derive(Clone,Debug)]
pub struct Startup {
    pub backend : BackendService,
    /// The project name we want to open on startup passed from the optional `--project` argument
    pub project_name : ProjectName
}

impl Startup {
    /// Provisional initial configuration that can be used during local deployments.
    pub fn new_local() -> FallibleResult<Startup> {
        let arguments    = ensogl::system::web::Arguments::new();
        let backend      = BackendService::from_web_arguments(&arguments)?;
        let project_name = arguments.get("project").map(ProjectName::new);
        let project_name = project_name.unwrap_or_else(|| {
            ProjectName::new(constants::DEFAULT_PROJECT_NAME)
        });
        Ok(Startup{backend,project_name})
    }
}
