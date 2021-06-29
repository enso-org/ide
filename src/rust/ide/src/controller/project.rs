//! A Project Controller.

use crate::prelude::*;

use crate::controller::graph::executed::Notification as GraphNotification;
use crate::controller::ide::StatusNotificationPublisher;
use crate::double_representation::project;

use enso_frp::web::platform;
use enso_frp::web::platform::Platform;
use enso_protocol::language_server::MethodPointer;
use enso_protocol::language_server::Path;
use parser::Parser;



// =================
// === Constants ===
// =================

/// The label of compiling stdlib message process.
pub const COMPILING_STDLIB_LABEL:&str = "Compiling standard library. It can take up to 1 minute.";

/// The requirements for Engine's version, in format understandable by
/// [`semver::VersionReq::parse`].
pub const ENGINE_VERSION_SUPPORTED        : &str = "^0.2.12";

/// The Engine version used in projects created in IDE.
// Usually it is a good idea to synchronize this version with the bundled Engine version in
// src/js/lib/project-manager/src/build.ts. See also https://github.com/enso-org/ide/issues/1359
pub const ENGINE_VERSION_FOR_NEW_PROJECTS : &str = "0.2.12";

/// The name of the module initially opened in the project view.
///
/// Currently, this name is hardcoded in the engine services and is populated for each project
/// created using engine's Project Picker service.
pub const INITIAL_MODULE_NAME:&str = "Main";

/// Name of the main definition.
///
/// This is the definition whose graph will be opened on IDE start.
pub const MAIN_DEFINITION_NAME:&str = "main";

/// The code with definition of the default `main` method.
pub fn default_main_method_code() -> String {
    format!(r#"{} = "Hello, World!""#, MAIN_DEFINITION_NAME)
}

/// The default content of the newly created initial main module file.
pub fn default_main_module_code() -> String {
    default_main_method_code()
}

/// Method pointer that described the main method, i.e. the method that project view wants to open
/// and which presence is currently required.
pub fn main_method_ptr
(project_name:project::QualifiedName, module_path:&model::module::Path) -> MethodPointer {
    module_path.method_pointer(project_name,MAIN_DEFINITION_NAME)
}



// =================
// === Utilities ===
// =================

/// Returns the path to package.yaml file for given project.
pub fn package_yaml_path(project_name:&str) -> String {
    match platform::current() {
        Some(Platform::Linux)   |
        Some(Platform::MacOS)   => format!("~/enso/projects/{}/package.yaml",project_name),
        Some(Platform::Windows) =>
            format!("%userprofile%\\enso\\projects\\{}\\package.yaml",project_name),
        _ => format!("<path-to-enso-projects>/{}/package.yaml",project_name)
    }
}


// ==============
// === Handle ===
// ==============

// === SetupResult ===

/// The result of initial project setup, containing handy controllers to be used in the initial
/// view.
#[derive(Clone,CloneRef,Debug)]
pub struct InitializationResult {
    /// The Text Controller for Main module code to be displayed in Code Editor.
    pub main_module_text:controller::Text,
    /// The Graph Controller for main definition's graph, to be displayed in Graph Editor.
    pub main_graph:controller::ExecutedGraph,
}

/// Project Controller Handle.
///
/// This controller supports IDE-related operations on a specific project.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct Project {
    pub logger               : Logger,
    pub model                : model::Project,
    pub status_notifications : StatusNotificationPublisher,
}

impl Project {
    /// Create a controller of given project.
    pub fn new(model:model::Project, status_notifications:StatusNotificationPublisher) -> Self {
        let logger = Logger::new("controller::Project");
        Self {logger,model,status_notifications}
    }

    /// Do the initial setup of opened project.
    ///
    /// This function should be called always after opening a new project in IDE. It checks if main
    /// module and main method are present in the project, and recreates them if missing.
    /// It also sends status notifications and warnings about the opened project (like
    /// warning about unsupported engine version).
    ///
    /// Returns the controllers of module and graph which should be displayed in the view.
    pub async fn initialize(&self) -> FallibleResult<InitializationResult> {
        let project     = self.model.clone_ref();
        let parser      = self.model.parser();
        let module_path = self.initial_module_path()?;
        let file_path   = module_path.file_path().clone();

        // TODO [mwu] This solution to recreate missing main file should be considered provisional
        //   until proper decision is made. See: https://github.com/enso-org/enso/issues/1050
        self.recreate_if_missing(&file_path,default_main_method_code()).await?;
        let method = main_method_ptr(project.qualified_name(),&module_path);
        let module = self.model.module(module_path).await?;
        Self::add_main_if_missing(project.qualified_name(),&module,&method,&parser)?;

        // Here, we should be relatively certain (except race conditions in case of multiple
        // clients that we currently do not support) that main module exists and contains main
        // method. Thus, we should be able to successfully create a graph controller for it.
        let main_module_text = controller::Text::new(&self.logger,&project,file_path).await?;
        let main_graph       = controller::ExecutedGraph::new(&self.logger,project,method).await?;

        self.notify_about_compiling_process(&main_graph);
        self.display_warning_on_unsupported_engine_version()?;

        Ok(InitializationResult {main_module_text,main_graph})
    }
}


// === Project Initialization Utilities ===

impl Project {
    /// Returns the path to the initially opened module in the given project.
    fn initial_module_path(&self) -> FallibleResult<model::module::Path> {
        crate::ide::initial_module_path(&self.model)
    }

    /// Create a file with default content if it does not already exist.
    pub async fn recreate_if_missing(&self, path:&Path, default_content:String) -> FallibleResult {
        let rpc = self.model.json_rpc();
        if !rpc.file_exists(path).await?.exists {
            rpc.write_file(path,&default_content).await?;
        }
        Ok(())
    }

    /// Add main method definition to the given module, if the method is not already defined.
    ///
    /// The lookup will be done using the given `main_ptr` value.
    pub fn add_main_if_missing
    (project_name:project::QualifiedName, module:&model::Module, main_ptr:&MethodPointer, parser:&Parser)
     -> FallibleResult {
        if module.lookup_method(project_name,main_ptr).is_err() {
            let mut info  = module.info();
            let main_code = default_main_method_code();
            let main_ast  = parser.parse_line(main_code)?;
            info.add_ast(main_ast,double_representation::module::Placement::End)?;
            module.update_ast(info.ast)?;
        }
        Ok(())
    }

    fn notify_about_compiling_process(&self, graph:&controller::ExecutedGraph) {
        let status_notif             = self.status_notifications.clone_ref();
        let compiling_process        = status_notif.publish_background_task(COMPILING_STDLIB_LABEL);
        let notifications            = graph.subscribe();
        let mut computed_value_notif = notifications.filter(|notification|
            futures::future::ready(matches!(notification, GraphNotification::ComputedValueInfo(_)))
        );
        executor::global::spawn(async move {
            computed_value_notif.next().await;
            status_notif.published_background_task_finished(compiling_process);
        });
    }

    fn display_warning_on_unsupported_engine_version(&self) -> FallibleResult {
        let requirements = semver::VersionReq::parse(ENGINE_VERSION_SUPPORTED)?;
        let version      = self.model.engine_version();
        if !requirements.matches(&version) {
            let message = format!("Unsupported Engine version. Please update engine_version in {} \
                to {}.",package_yaml_path(&self.model.name()),ENGINE_VERSION_FOR_NEW_PROJECTS);
            self.status_notifications.publish_event(message);
        }
        Ok(())
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;

    #[test]
    fn new_project_engine_version_fills_requirements() {
        let requirements = semver::VersionReq::parse(ENGINE_VERSION_SUPPORTED).unwrap();
        let version      = semver::Version::parse(ENGINE_VERSION_FOR_NEW_PROJECTS).unwrap();
        assert!(requirements.matches(&version))
    }

    #[wasm_bindgen_test]
    fn adding_missing_main() {
        let _ctx        = TestWithLocalPoolExecutor::set_up();
        let parser      = parser::Parser::new_or_panic();
        let mut data    = crate::test::mock::Unified::new();
        let module_name = data.module_path.module_name();
        let main_ptr    = main_method_ptr(data.project_name.clone(),&data.module_path);

        // Check that module without main gets it after the call.
        let empty_module_code = "";
        data.set_code(empty_module_code);
        let urm    = data.undo_redo_manager();
        let module = data.module(urm.clone_ref());
        assert!(module.lookup_method(data.project_name.clone(),&main_ptr).is_err());
        Project::add_main_if_missing(data.project_name.clone(), &module, &main_ptr, &parser).unwrap();
        assert!(module.lookup_method(data.project_name.clone(),&main_ptr).is_ok());

        // Now check that modules that have main already defined won't get modified.
        let mut expect_intact = move |code:&str| {
            data.set_code(code);
            let module = data.module(urm.clone_ref());
            Project::add_main_if_missing(data.project_name.clone(), &module, &main_ptr, &parser).unwrap();
            assert_eq!(code,module.ast().repr());
        };
        expect_intact("main = 5");
        expect_intact("here.main = 5");
        expect_intact(&format!("{}.main = 5",module_name));
    }
}
