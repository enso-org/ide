use crate::prelude::*;
use crate::controller::ide::ManagingProjectAPI;

use enso_protocol::project_manager::ProjectName;


pub struct Handle {
    pub logger  : Logger,
    pub project : model::Project,
}

impl Handle {
    async fn new(project_name:ProjectName, json_endpoint:String, binary_endpoint:String) -> FallibleResult<Self> {
        let logger          = Logger::new();
        // TODO[ao]: we should think how to handle engine's versions in cloud.
        //     https://github.com/enso-org/ide/issues/1195
        // let version         = ENGINE_VERSION_FOR_NEW_PROJECTS.to_owned();
        let project_id      = default();
        let project         = model::project::Synchronized::new_connected(logger, None, json_endpoint, binary_endpoint, project_id, project_name).await?;
        Ok(Self{logger,project})
    }
}

impl controller::ide::API for Handle {
    fn current_project(&self) -> model::Project {
        self.project.clone_ref()
    }

    fn manage_projects(&self) -> Option<&dyn ManagingProjectAPI> {
        None
    }
}

