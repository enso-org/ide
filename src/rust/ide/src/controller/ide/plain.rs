//! The Plain IDE Controller
//!
//! See [`crate::controller::ide`] for more detailed description of IDE Controller API.

use crate::prelude::*;
use crate::controller::ide::ManagingProjectAPI;
use crate::controller::ide::Notification;
use crate::controller::ide::StatusNotificationPublisher;

use enso_protocol::project_manager::ProjectName;
use parser::Parser;



// =============
// === Error ===
// =============

#[allow(missing_docs)]
#[fail(display="Project operations are not supported.")]
#[derive(Copy,Clone,Debug,Fail)]
pub struct ProjectOperationsNotSupported;



// ===============================
// === Plain Controller Handle ===
// ===============================

/// Plain IDE Controller Handle.
///
/// The Plain Controller does not allow for managing projects: it has the single project model
/// as a project opened in IDE which does not change (it is set up during construction).
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct Handle {
    pub logger               : Logger,
    pub status_notifications : StatusNotificationPublisher,
    pub parser               : Parser,
    pub project              : model::Project,
}

impl Handle {
    /// Create IDE Controller for a given opened project.
    pub fn new(project:model::Project) -> Self {
        let logger = Logger::new("controller::ide::Plain");
        let status_notifications = default();
        let parser               = Parser::new_or_panic();
        Self {logger,status_notifications,parser,project}
    }

    /// Create IDE Controller from Language Server endpoints, describing the opened project.
    pub async fn from_ls_endpoints
    ( project_name    : ProjectName
    , version         : semver::Version
    , json_endpoint   : String
    , binary_endpoint : String
    ) -> FallibleResult<Self> {
        let logger     = Logger::new("controller::ide::Plain");
        //TODO [ao]: this should be not the default; instead project model should not need the id.
        //    See also https://github.com/enso-org/ide/issues/1572
        let project_id = default();
        let project    = model::project::Synchronized::new_connected
            (&logger,None,json_endpoint,binary_endpoint,version,project_id,project_name).await?;
        let status_notifications = default();
        let parser               = Parser::new_or_panic();
        Ok(Self{logger,project,status_notifications,parser})
    }
}

impl controller::ide::API for Handle {
    fn current_project     (&self) -> model::Project               { self.project.clone_ref()   }
    fn status_notifications(&self) -> &StatusNotificationPublisher { &self.status_notifications }
    fn parser              (&self) -> &Parser                      { &self.parser               }

    fn subscribe(&self) -> StaticBoxStream<Notification> {
        futures::stream::empty().boxed_local()
    }

    fn manage_projects(&self) -> FallibleResult<&dyn ManagingProjectAPI> {
        Err(ProjectOperationsNotSupported.into())
    }
}
