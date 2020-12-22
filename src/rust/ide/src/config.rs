//! This module provides IDE configuration structures.

use crate::prelude::*;
use crate::constants;

use enso_protocol::project_manager::ProjectName;
use ensogl::system::web;



// ==============
// === Config ===
// ==============

/// The path at which the config is accessible. This needs to be synchronised with the
/// `src/config.yaml` configuration file. In the future, we could write a procedural macro, which
/// loads the configuration and splits Rust variables from it during compilation time. This is not
/// possible by using macro rules, as there is no way to plug in the output of `include_str!` macro
/// to another macro input.
const WINDOW_CFG_PATH : &[&str] = &["enso","config"];

/// Defines a new config structure. The provided fields are converted to optional fields. The config
/// constructor queries JavaScript configuration for the keys defined in this structure. For each
/// resulting string value, it converts it to the defined type. It also reports warnings for all
/// config options that were provided, but were not matched this definition.
macro_rules! define_config {
    ($name:ident { $($field:ident : $field_type:ty),* $(,)? }) => {
        /// The structure containing application configs.
        #[derive(Clone,Debug,Default)]
        pub struct $name {
            $($field : Option<$field_type>),*
        }

        impl $name {
            /// Constructor.
            pub fn new() -> Self {
                let logger = Logger::new(stringify!{$name});
                let window = web::window();
                match web::reflect_get_nested_object(&window,WINDOW_CFG_PATH).ok() {
                    None => {
                        let path = WINDOW_CFG_PATH.join(".");
                        error!(&logger,"The config path '{path}' is invalid.");
                        default()
                    }
                    Some(cfg) => {
                        let keys     = web::object_keys(&cfg);
                        let mut keys = keys.into_iter().collect::<HashSet<String>>();
                        $(
                            let name   = stringify!{$field};
                            let $field = web::reflect_get_nested_string(&cfg,&[name]).ok();
                            let $field = $field.map(|t|t.into());
                            keys.remove(name);
                        )*
                        for key in keys {
                            warning!(&logger,"Unknown config option provided '{key}'.");
                        }
                        Self {$($field),*}
                    }
                }
            }
        }
    };
}

define_config! {
    ConfigReader {
        entry   : String,
        project : ProjectName,
    }
}



// ===============
// === Startup ===
// ===============

/// Configuration data necessary to initialize IDE.
///
/// We will eventually want to load it from a configuration file.
#[derive(Clone,Debug)]
pub struct Startup {
    /// WebSocket endpoint of the project manager service.
    pub project_manager_endpoint : String,
    /// The project name we want to open on startup passed from the optional `--project` argument
    pub project_name : ProjectName
}

impl Startup {
    /// Provisional initial configuration that can be used during local deployments.
    pub fn new_local() -> Startup {
        let config                   = ConfigReader::new();
        let project_manager_endpoint = constants::PROJECT_MANAGER_ENDPOINT.into();
        let project_name             = config.project.unwrap_or_else(|| {
            ProjectName::new(constants::DEFAULT_PROJECT_NAME)
        });
        Startup{project_manager_endpoint,project_name}
    }
}
