pub mod desktop;
pub mod cloud;

use crate::prelude::*;

use crate::notification;

use flo_stream::Subscriber;



// ============================
// === Status Notifications ===
// ============================

pub type ProcessHandle = usize;

#[derive(Clone,Debug)]
pub enum StatusNotification {
    Event           { label:String                       },
    ProcessStarted  { label:String, handle:ProcessHandle },
    ProcessFinished {               handle:ProcessHandle },
}

#[derive(Clone,CloneRef,Debug,Default)]
pub struct StatusNotifications {
    publisher           : notification::Publisher<StatusNotification>,
    next_process_handle : Rc<Cell<usize>>,
}

impl StatusNotifications {
    pub fn new() -> Self { default() }

    pub fn publish_event(&self, label:impl Into<String>) {
        let label        = label.into();
        let notification = StatusNotification::Event {label};
        executor::global::spawn(self.publisher.publish(notification));
    }

    pub fn publish_process(&self, label:impl Into<String>) -> ProcessHandle {
        let label  = label.into();
        let handle = self.next_process_handle.get();
        self.next_process_handle.set(handle + 1);
        let notification = StatusNotification::ProcessStarted {label,handle};
        executor::global::spawn(self.publisher.publish(notification));
        handle
    }

    pub fn published_process_finished(&self, handle:ProcessHandle) {
        let notification = StatusNotification::ProcessFinished {handle};
        executor::global::spawn(self.publisher.publish(notification));
    }

    pub fn subscribe(&self) -> impl futures::Stream<Item=StatusNotification> {
        self.publisher.subscribe()
    }
}



// ===========
// === API ===
// ===========

pub trait ManagingProjectAPI {
    fn create_new_project<'a>(&'a self) -> BoxFuture<'a, FallibleResult>;
}

pub trait API:Debug {
    fn current_project(&self) -> model::Project;

    fn status_notifications(&self) -> &StatusNotifications;

    fn manage_projects(&self) -> Option<&dyn ManagingProjectAPI>;
}

pub type Handle  = Rc<dyn API>;
pub type Desktop = desktop::Handle;
pub type Cloud   = cloud::Handle;
