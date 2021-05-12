use crate::prelude::*;
use crate::controller::ide::{ManagingProjectAPI, Notification};
use crate::controller::ide::StatusNotifications;

use enso_protocol::project_manager::ProjectName;
use flo_stream::Subscriber;



#[derive(Clone,CloneRef,Debug)]
pub struct Handle {
    pub logger               : Logger,
    pub status_notifications : StatusNotifications,
    pub project              : model::Project,
}

impl Handle {
    pub async fn new
    (project_name:ProjectName, json_endpoint:String, binary_endpoint:String)
    -> FallibleResult<Self> {
        let logger = Logger::new("controller::ide::Cloud");
        // TODO[ao]: we should think how to handle engine's versions in cloud.
        //     https://github.com/enso-org/ide/issues/1195
        let version              = semver::Version::parse(controller::project::ENGINE_VERSION_FOR_NEW_PROJECTS)?;
        let project_id           = default();
        let project              = model::project::Synchronized::new_connected
            (&logger,None,json_endpoint,binary_endpoint,version,project_id,project_name).await?;
        let status_notifications = default();
        Ok(Self{logger,project,status_notifications})
    }
}

impl controller::ide::API for Handle {
    fn current_project(&self) -> model::Project {
        self.project.clone_ref()
    }

    fn status_notifications(&self) -> &StatusNotifications { &self.status_notifications }

    fn subscribe(&self) -> StaticBoxStream<Notification> {
        futures::stream::empty().boxed_local()
    }

    fn manage_projects(&self) -> Option<&dyn ManagingProjectAPI> {
        None
    }
}

