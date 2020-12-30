//! This module provides IDE configuration structures.
use crate::prelude::*;

use crate::constants;

use enso_protocol::project_manager::ProjectName;
use ensogl::system::web;



// =================
// === ArgReader ===
// =================

/// Marker trait used to disambiguate overlapping impls of [`ArgReader`].
pub trait ArgMarker {}

/// Trait used to convert provided string arguments to the desired type.
#[allow(missing_docs)]
pub trait ArgReader : Sized {
    fn read_arg(str:String) -> Option<Self>;
}


// === Default ===

/// Helper trait used to disambiguate overlapping impls of [`ArgReader`].
#[allow(missing_docs)]
pub trait ArgReaderFromString : Sized {
    fn read_arg_from_string(str:String) -> Option<Self>;
}

impl<T> ArgReaderFromString for T
where String:TryInto<T> {
    fn read_arg_from_string(str:String) -> Option<Self> {
        str.try_into().ok()
    }
}

impl<T> ArgReaderFromString for T {
    default fn read_arg_from_string(_:String) -> Option<Self> {
        unreachable!()
    }
}

impl<T> ArgMarker for T where T : TryFrom<String> {}
impl<T> ArgReader for T where T : ArgMarker {
    default fn read_arg(str:String) -> Option<Self> {
        ArgReaderFromString::read_arg_from_string(str)
    }
}


// === Specializations ===

impl ArgMarker for bool {}
impl ArgReader for bool {
    fn read_arg(str:String) -> Option<Self> {
        match &str[..] {
            "true"     => Some(true),
            "false"    => Some(false),
            "ok"       => Some(true),
            "fail"     => Some(false),
            "enabled"  => Some(true),
            "disabled" => Some(false),
            "yes"      => Some(true),
            "no"       => Some(false),
            _          => None,
        }
    }
}



// =================
// === read_args ===
// =================

/// Defines an application argument reader. As a result, a new lazy-initialized static variable
/// `ARGS` will be created and it will read the arguments on its first access (in case you want to
/// force argument read, use the `init` function).
///
/// For example, given the following definition:
/// ```
/// read_args! {
///     js::global.config {
///         entry      : String,
///         project    : String,
///         dark_theme : bool,
///     }
/// }
/// ```
///
/// The following structs will be generated (some functions omitted for clarity):
///
/// ```
/// #[derive(Clone,Copy,Debug)]
/// pub struct ArgNames {
///     entry      : &'static str,
///     project    : &'static str,
///     dark_theme : &'static str,
/// }
///
/// #[derive(Clone, Debug, Default)]
/// pub struct Args {
///     __names__  : ArgNames,
///     entry      : Option<String>,
///     project    : Option<String>,
///     dark_theme : Option<bool>,
/// }
///
/// lazy_static! {
///     pub static ref ARGS : Args = Args::new();
/// }
/// ```
///
/// The header `js::global.config` means that the JavaScript space will be queried for variable
/// `global.config`, which will be queried for every field of the generated structure. In case the
/// JavaScript variable will not contain the key, it will be left as None. For each available key,
/// the [`ArgReader`] trait will be used to read it back to Rust types. The [`ArgReader`] is a thin
/// wrapper over the [`Into`] trait with some additional conversions (e.g. for [`bool`]). In case
/// the conversion will fail, a warning will be raised.
macro_rules! read_args {
    (js::$($path:ident).* { $($field:ident : $field_type:ty),* $(,)? }) => {

        /// Reflection mechanism containing string representation of option names.
        #[derive(Clone,Copy,Debug)]
        pub struct ArgNames {
            $($field : &'static str),*
        }

        impl Default for ArgNames {
            fn default() -> Self {
                $(let $field = stringify!{$field};)*
                Self {$($field),*}
            }
        }

        /// The structure containing application configs.
        #[derive(Clone,Debug,Default)]
        pub struct Args {
            __names__ : ArgNames,
            $($field : Option<$field_type>),*
        }

        impl Args {
            /// Constructor.
            fn new() -> Self {
                let logger = Logger::new(stringify!{Args});
                let window = web::window();
                let path   = vec![$(stringify!($path)),*];
                match web::reflect_get_nested_object(&window,&path).ok() {
                    None => {
                        let path = path.join(".");
                        error!(&logger,"The config path '{path}' is invalid.");
                        default()
                    }
                    Some(cfg) => {
                        let __names__ = default();
                        let keys      = web::object_keys(&cfg);
                        let mut keys  = keys.into_iter().collect::<HashSet<String>>();
                        $(
                            let name   = stringify!{$field};
                            let tp     = stringify!{$field_type};
                            let $field = web::reflect_get_nested_string(&cfg,&[name]).ok();
                            let $field = $field.map(ArgReader::read_arg);
                            if $field == Some(None) {
                                warning!(&logger,"Failed to convert the argument '{name}' value \
                                                  to the '{tp}' type.");
                            }
                            let $field = $field.flatten();
                            keys.remove(name);
                        )*
                        for key in keys {
                            warning!(&logger,"Unknown config option provided '{key}'.");
                        }
                        Self {__names__,$($field),*}
                    }
                }
            }

            /// This is a dummy function which initializes the arg reading process. This function
            /// does nothing, however, in order to call it, the user would need to access a field in
            /// the lazy static variable `ARGS`, which would trigger argument parsing process.
            pub fn init(&self) {}

            /// Reflection mechanism to get string representation of argument names.
            pub fn names(&self) -> &ArgNames {
                &self.__names__
            }
        }

        lazy_static! {
            /// Application arguments initialized in a lazy way (on first read).
            pub static ref ARGS : Args = Args::new();
        }
    };
}



// ============
// === Args ===
// ============

/// Please note that the path at which the config is accessible (`enso.config`) is hardcoded below.
/// This needs to be synchronised with the `src/config.yaml` configuration file. In the future, we
/// could write a procedural macro, which loads the configuration and splits Rust variables from it
/// during compilation time. This is not possible by using macro rules, as there is no way to plug
/// in the output of `include_str!` macro to another macro input.
read_args! {
    js::enso.config {
        entry                : String,
        project              : ProjectName,
        project_manager      : String,
        language_server_rpc  : String,
        language_server_data : String,
        platform             : web::platform::Platform,
        frame                : bool,
        dark_theme           : bool,
        high_contrast        : bool,
    }
}



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
    pub fn from_web_arguments(config:&Args) -> FallibleResult<Self> {
        if let Some(endpoint) = &config.project_manager {
            if config.language_server_rpc.is_some() || config.language_server_data.is_some() {
                Err(MutuallyExclusiveOptions.into())
            } else {
                let endpoint = endpoint.clone();
                Ok(Self::ProjectManager {endpoint})
            }
        } else {
            match (&config.language_server_rpc,&config.language_server_data) {
                (Some(json_endpoint),Some(binary_endpoint)) => {
                    let json_endpoint   = json_endpoint.clone();
                    let binary_endpoint = binary_endpoint.clone();
                    Ok(Self::LanguageServer {json_endpoint,binary_endpoint})
                }
                (None,None)    => Ok(default()),
                (Some(_),None) => Err(MissingOption(config.names().language_server_data).into()),
                (None,Some(_)) => Err(MissingOption(config.names().language_server_rpc).into())
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
        let project_name = ARGS.project.clone().unwrap_or_else(||
            ProjectName::new(constants::DEFAULT_PROJECT_NAME)
        );
        Ok(Startup{backend,project_name})
    }
}
