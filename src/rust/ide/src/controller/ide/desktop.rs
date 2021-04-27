use crate::prelude::*;



use enso_protocol::project_manager;
use crate::controller::ide::{API, ManagingProjectAPI, Notification};
use crate::notification::Subscriber;
use crate::notification::Publisher;
use futures::future::LocalBoxFuture;


pub struct Handle {
    current_project : Rc<CloneRefCell<model::Project>>,
    project_manager : Rc<dyn project_manager::API>,
    notifications   : Publisher<Notification>,
}

impl Handle {
    fn new(project_manager:Rc<dyn project_manager::API>, initial_project:model::Project) -> Self {
        let notifications   = default();
        let current_project = initial_project;
        Self {current_project,project_manager,notifications}
    }
}

impl API for Handle {
    fn current_project(&self) -> model::Project {
        self.current_project.get()
    }

    fn subscribe(&self) -> Subscriber<Notification> {
        self.notifications.subscribe()
    }

    fn manage_projects(&self) -> Option<&dyn ManagingProjectAPI> {
        Some(self)
    }
}

impl ManagingProjectAPI for Handle {
    fn create_new_project<'a>(&'a self) -> BoxFuture<FallibleResult> {

    }
}
