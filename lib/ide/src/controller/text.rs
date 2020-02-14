//! Text Controller module.
//!
//! Facade over filesystem API or module text API for text editor. Does discerning between Luna
//! module file and plain text file. In case of luna module idmap and metadata are hidden for the
//! user.

use crate::prelude::*;

use crate::controller::*;

use failure::_core::fmt::{Formatter, Error};
use flo_stream::{Publisher, Subscriber};
use flo_stream::MessagePublisher;
use file_manager_client as fmc;
use json_rpc::error::RpcError;
use shapely::shared;
use std::ops::Range;



// ====================
// === Notification ===
// ====================

/// A notification from TextController.
#[derive(Clone,Debug)]
pub enum Notification {
    /// File contents needs to be set to the following due to synchronization with external state.
    SetNewContent(String),
}



// =======================
// === Text Controller ===
// =======================

shared! { ControllerHandle

    /// Data stored by the text controller.
    pub struct State {
        /// A path to file the controller is handling.
        file_path : fmc::Path,
        /// A module handle in case the TextController is managing Luna module file.
        module: Option<module::ControllerHandle>,
        /// Sink where we put events to be consumed by the view.
        notification_publisher: Publisher<Notification>,
        /// File manager handle, used to obtain file information in case of plain text file.
        file_manager: file::Handle,
    }

    impl {
        /// Create controller managing plain text file.
        pub fn new(path:fmc::Path, file_manager:file::Handle) -> Self {
            Self {file_manager,
                file_path              : path,
                module                 : None,
                notification_publisher : Publisher::new(10),
            }
        }

        /// Get subscriber receiving controller's notifications.
        pub fn subscribe(&mut self) -> Subscriber<Notification> {
            self.notification_publisher.subscribe()
        }
    }
}

impl ControllerHandle {
    /// Create controller managing Luna module file.
    pub fn new_for_module(module:module::ControllerHandle, file_manager:file::Handle) -> Self {
        let file_path = module.location_as_path();
        let mut state = State::new(file_path, file_manager);
        state.module = Some(module);
        Self { rc:Rc::new(RefCell::new(state)) }
    }

    /// Read file's content.
    pub fn read_content(&self) -> impl Future<Output=Result<String,RpcError>> {
        let (file_manager,path) = self.with_borrowed(|state| {
            (state.file_manager.clone(), state.file_path.clone())
        });
        file_manager.read(path)
    }
}

// === Debug implementations ===

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f,"Text Controller {{ file_path: {:?}, module: {:?} }}",self.file_path,self.module)
    }
}

// TODO[ao] why the shared macro cannot derive this?
impl Debug for ControllerHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.rc.borrow().fmt(f)
    }
}
