use crate::prelude::*;

use crate::controller::*;

use failure::_core::fmt::{Formatter, Error};
use flo_stream::{Publisher, Subscriber};
use flo_stream::MessagePublisher;
use file_manager_client as fmc;
use json_rpc::error::RpcError;
use shapely::shared;
use std::ops::Range;


#[derive(Clone,Debug)]
pub enum Notification {
    /// File contents needs to be set to the following due to
    /// synchronization with external state.
    SetNewContent(String),
}

/// Edit action on the text document that replaces text on given span with
/// a new one.
#[derive(Clone,Debug)]
pub struct Edit {
    /// Replaced range
    pub replace  : Range<usize>,
    /// Text to be placed. May be empty to erase portion of text.
    pub new_text : String,
}

shared! { ControllerHandle

    /// Data stored by the text controller.
    pub struct State {
        file_path : fmc::Path,
        /// A module handle in case the TextController is managing Luna module file.
        module: Option<module::ControllerHandle>,
        /// Sink where we put events to be consumed by the view.
        notification_publisher: Publisher<Notification>,

        file_manager: file::Handle,
    }

    impl {
        pub fn new(path:fmc::Path, file_manager:file::Handle) -> Self {
            Self {file_manager,
                file_path              : path,
                module                 : None,
                notification_publisher : Publisher::new(10),
            }
        }

        pub fn subscribe(&mut self) -> Subscriber<Notification> {
            self.notification_publisher.subscribe()
        }
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Text Controller for module {:?}", self.module)
    }
}

// TODO[ao] why the shared macro cannot derive this?
impl Debug for ControllerHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.rc.borrow().fmt(f)
    }
}

impl ControllerHandle {
    pub fn new_for_module(module:module::ControllerHandle, file_manager:file::Handle) -> Self {
        let file_path = module.location_as_path();
        let mut state = State::new(file_path, file_manager);
        state.module = Some(module);
        Self { rc:Rc::new(RefCell::new(state)) }
    }

    pub fn read_content(&self) -> impl Future<Output=Result<String,RpcError>> {
        let (file_manager,path) = self.with_borrowed(|state| {
            (state.file_manager.clone(), state.file_path.clone())
        });
        file_manager.read(path)
    }
}
