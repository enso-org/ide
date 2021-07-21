//! This module provides IDE configuration structures.
use crate::prelude::*;

use crate::constants;

use enso_protocol::project_manager::ProjectName;
use enso_args::Args;
use enso_args::ARGS;



// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Missing program option: {}.",0)]
pub struct MissingOption (&'static str);

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Provided options for both project manager and language server connection.")]
pub struct MutuallyExclusiveOptions;



// ======================
// === BackendService ===
// ======================

/// A Configuration defining to what backend service should IDE connect.
#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub enum BackendService {
    /// Connect to the project manager. Using the project manager IDE will open or create a specific
    /// project and connect to its Language Server.
    ProjectManager {endpoint:String},
    /// Connect to the language server of some project. The project managing operations will be
    /// unavailable.
    LanguageServer {
        json_endpoint   : String,
        binary_endpoint : String,
        namespace       : String,
    }
}

impl Default for BackendService {
    fn default() -> Self {
        Self::ProjectManager {endpoint:constants::PROJECT_MANAGER_ENDPOINT.into()}
    }
}

impl BackendService {
    /// Read backend configuration from the web arguments. See also [`web::Arguments`]
    /// documentation.
    pub fn from_web_arguments(args:&Args) -> FallibleResult<Self> {
        if let Some(endpoint) = &args.project_manager {
            if args.language_server_rpc.is_some() || args.language_server_data.is_some() {
                Err(MutuallyExclusiveOptions.into())
            } else {
                let endpoint = endpoint.clone();
                Ok(Self::ProjectManager {endpoint})
            }
        } else {
            match (&args.language_server_rpc,&args.language_server_data,&args.namespace) {
                (Some(json_endpoint),Some(binary_endpoint),Some(namespace)) => {
                    let json_endpoint   = json_endpoint.clone();
                    let binary_endpoint = binary_endpoint.clone();
                    let namespace       = namespace.clone();
                    Ok(Self::LanguageServer {json_endpoint,binary_endpoint,namespace})
                }
                (None,None,None) => Ok(default()),
                (None,_,_)       => Err(MissingOption(args.names().language_server_rpc()).into()),
                (_,None,_)       => Err(MissingOption(args.names().language_server_data()).into()),
                (_,_,None)       => Err(MissingOption(args.names().namespace()).into()),
            }
        }
    }
}



// ===============
// === Startup ===
// ===============

/// Configuration data necessary to initialize IDE.
#[derive(Clone,Debug)]
pub struct Startup {
    /// The configuration of connection to the backend service.
    pub backend : BackendService,
    /// The project name we want to open on startup.
    pub project_name : ProjectName,
}

impl Default for Startup {
    fn default() -> Self {
        Self {
            backend      : default(),
            project_name : ProjectName(constants::DEFAULT_PROJECT_NAME.to_owned()),
        }
    }
}

impl Startup {
    /// Read configuration from the web arguments. See also [`web::Arguments`] documentation.
    pub fn from_web_arguments() -> FallibleResult<Startup> {
        let backend      = BackendService::from_web_arguments(&ARGS)?;
        let project_name = ARGS.project.clone().map(|t|t.into()).unwrap_or_else(||
            ProjectName::new(constants::DEFAULT_PROJECT_NAME)
        );
        Ok(Startup{backend,project_name})
    }
}
