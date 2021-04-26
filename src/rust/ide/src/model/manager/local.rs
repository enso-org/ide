use crate::prelude::*;

use crate::model::manager::{ProjectId, ENGINE_VERSION_FOR_NEW_PROJECTS, ManagingAPI, create_project_model};
use enso_protocol::{project_manager, language_server, binary};
use enso_protocol::project_manager::{ProjectName, MissingComponentAction};
use crate::constants::UNNAMED_PROJECT_NAME;
use crate::transport::web::WebSocket;
use crate::model::Project;

#[derive(Clone,CloneRef,Debug)]
pub struct Manager {
    logger          : Logger,
    current_project : Option<ProjectId>,
    project_manager : Rc<dyn project_manager::API>
}

impl Manager {
    fn new(project_manager: Rc<dyn project_manager::API>) -> Self {
        let logger          = Logger::new("local::Manager");
        let current_project = default();
        Self {logger,current_project,project_manager}
    }
}

impl model::manager::ManagingAPI for Manager {
    fn create_new_unnamed_project<'a>(&'a self) -> BoxFuture<'a, FallibleResult<model::Project>> {
        async {
            let list = self.project_manager.list_projects(&None).await?;
            let names:HashSet<ProjectName> = list.projects.into_iter().map(|p| p.name).collect();
            let candidates_with_suffix = (1..).map(|i| format!("{}_{}", UNNAMED_PROJECT_NAME, i));
            let candidates = std::iter::once(UNNAMED_PROJECT_NAME.to_owned()).chain(candidates_with_suffix);
            let candidates = candidates.map(ProjectName);
            let name       = candidates.skip_while(|c| names.contains(&c)).next().unwrap();
            let version    = Some(ENGINE_VERSION_FOR_NEW_PROJECTS.to_owned());
            let action     = MissingComponentAction::Install;

            let new_project = self.project_manager.create_project(name.deref(),&version,&action).await?.project_id;
            self.open_project(new_project,name).await
        }
    }
}

impl model::manager::API for Manager {
    fn initial_project<'a>(&'a self) -> BoxFuture<'a, FallibleResult<model::Project>> {
        self.create_new_unnamed_project()
    }

    fn manage_projects(&self) -> Option<&dyn ManagingAPI> { Some(self) }
}

impl Manager {

    async fn open_project(self, id:ProjectId, name:ProjectName) -> FallibleResult<model::Project> {
        use project_manager::MissingComponentAction::*;

        let opened_project  = self.project_manager.open_project(&id,&Install).await?;
        let logger          = &self.logger;
        let json_endpoint   = opened_project.language_server_json_address.to_string();
        let binary_endpoint = opened_project.language_server_binary_address.to_string();
        let engine_version  = opened_project.engine_version;
        let manager         = Some(self.project_manager.clone_ref());
        create_project_model(logger,manager,json_endpoint,binary_endpoint,engine_version,id,name)
            .await
    }
}
