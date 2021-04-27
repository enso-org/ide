pub mod desktop;
pub mod cloud;

use crate::prelude::*;

use flo_stream::Subscriber;



// =====================
// === Notifications ===
// =====================

pub enum Notification {
    NewOpenedProject
}

pub trait ManagingProjectAPI {
    fn create_new_project<'a>(&'a self) -> BoxFuture<'a, FallibleResult>;
}

pub trait API {
    fn current_project(&self) -> model::Project;

    fn subscribe(&self) -> Subscriber<Notification>;

    fn manage_projects(&self) -> Option<&dyn ManagingProjectAPI>;
}

pub type Handle = Rc<dyn API>;
