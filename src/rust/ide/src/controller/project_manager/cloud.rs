use crate::prelude::*;

use crate::controller::project_manager::create_project_model;
use crate::controller::project_manager::ENGINE_VERSION_FOR_NEW_PROJECTS;

use futures::future::LocalBoxFuture;

#[derive(Clone,CloneRef,Debug)]
pub struct ProjectManager {
    logger          : Logger,
    json_endpoint   : ImString,
    binary_endpoint : ImString,
}

impl ProjectManager {
    pub fn new(json_endpoint:impl Into<ImString>, binary_endpoint:impl Into<ImString>) -> Self {
        Self {
            logger          : Logger::new("cloud::Manager"),
            json_endpoint   : json_endpoint.into(),
            binary_endpoint : binary_endpoint.into(),
        }
    }
}

impl controller::project_manager::API for ProjectManager {
    fn initial_project<'a>(&'a self) -> LocalBoxFuture<'a, FallibleResult<model::Project>> {
        let logger          = &self.logger;
        let project_manager = None;
        let json_endpoint   = json_endpoint.clone();
        let binary_endpoint = binary_endpoint.clone();
        // TODO[ao]: we should think how to handle engine's versions in cloud.
        //     https://github.com/enso-org/ide/issues/1195
        let version         = ENGINE_VERSION_FOR_NEW_PROJECTS.to_owned();
        let id              = default();
        let name            = self.config.project_name.clone();
        create_project_model(logger,project_manager,json_endpoint,binary_endpoint,version,id,name)
            .boxed()
    }

    fn manage_projects(&self) -> Option<&dyn model::project_manager::ManagingAPI> { None }
}
