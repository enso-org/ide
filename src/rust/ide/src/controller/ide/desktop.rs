use crate::prelude::*;

use enso_protocol::project_manager;
use crate::controller::ide::{API, ManagingProjectAPI, StatusNotifications};
use crate::notification::Publisher;
use crate::ide;

use flo_stream::Subscriber;
use futures::future::LocalBoxFuture;
use enso_protocol::project_manager::{ProjectName, MissingComponentAction};



// =================
// === Constants ===
// =================

const UNNAMED_PROJECT_NAME:&str = "Unnamed";



// =============================
// === The Controller Handle ===
// =============================

#[derive(Clone,CloneRef,Derivative)]
#[derivative(Debug)]
pub struct Handle {
    logger               : Logger,
    current_project      : Rc<CloneRefCell<model::Project>>,
    #[derivative(Debug="ignore")]
    project_manager      : Rc<dyn project_manager::API>,
    status_notifications : StatusNotifications,
}

impl Handle {
    pub fn new_with_project(project_manager:Rc<dyn project_manager::API>, initial_project:model::Project) -> Self {
        let logger               = Logger::new("controller::ide::Desktop");
        let current_project      = Rc::new(CloneRefCell::new(initial_project));
        let status_notifications = default();
        Self {logger,current_project,project_manager,status_notifications}
    }

    pub async fn new_with_opened_project(project_manager:Rc<dyn project_manager::API>, name:ProjectName) -> FallibleResult<Self> {
        let initializer = ide::initializer::WithProjectManager::new(Logger::new("Handle::new"),project_manager.clone_ref(),name);
        let model       = initializer.initialize_project_model().await?;
        Ok(Self::new_with_project(project_manager,model))
    }
}

impl API for Handle {
    fn current_project     (&self) -> model::Project       { self.current_project.get() }
    fn status_notifications(&self) -> &StatusNotifications { &self.status_notifications }

    fn manage_projects     (&self) -> Option<&dyn ManagingProjectAPI> {
        Some(self)
    }
}

impl ManagingProjectAPI for Handle {
    fn create_new_project<'a>(&'a self) -> BoxFuture<'a, FallibleResult> {
        async move {
            let list                       = self.project_manager.list_projects(&None).await?;
            let names:HashSet<ProjectName> = list.projects.into_iter().map(|p| p.name).collect();
            let candidates_with_suffix = (1..).map(|i| format!("{}_{}", UNNAMED_PROJECT_NAME, i));
            let candidates = std::iter::once(UNNAMED_PROJECT_NAME.to_owned()).chain(candidates_with_suffix);
            let candidates = candidates.map(ProjectName);
            let name       = candidates.skip_while(|c| names.contains(c)).next().unwrap();
            let version    = Some(controller::project::ENGINE_VERSION_FOR_NEW_PROJECTS.to_owned());
            let action     = MissingComponentAction::Install;

            let new_project = self.project_manager.create_project(name.deref(),&version,&action).await?.project_id;
            self.current_project.set(model::project::Synchronized::new_opened(&self.logger,self.project_manager.clone_ref(),new_project,name).await?);
            Ok(())
        }.boxed_local()
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    use crate::controller::project::ENGINE_VERSION_SUPPORTED;



    #[test]
    fn new_project_engine_version_fills_requirements() {
        let requirements = semver::VersionReq::parse(ENGINE_VERSION_SUPPORTED).unwrap();
        let version      = semver::Version::parse(ENGINE_VERSION_FOR_NEW_PROJECTS).unwrap();
        assert!(requirements.matches(&version))
    }
}
