use crate::prelude::*;

use utils::make_handles;
use crate::controller::*;

use std::ops::Range;
use flo_stream::{Publisher, Subscriber};
use flo_stream::MessagePublisher;
use failure::_core::fmt::{Formatter, Error};
use file_manager_client as fmc;
use json_rpc::error::RpcError;


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

/// Data stored by the text controller.
pub struct Data {
    file_path : fmc::Path,
    /// A module handle in case the TextController is managing Luna module file.
    module: Option<module::Handle>,
    /// Sink where we put events to be consumed by the view.
    notification_publisher: Publisher<Notification>,
    file_manager: file::Handle,

}

impl Data {
    fn new(path:fmc::Path, file_manager:file::Handle) -> Self {
        Data {file_manager,
            file_path              : path,
            module                 : None,
            notification_publisher : Publisher::new(10),
        }
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Text Controller for module {:?}", self.module)
    }
}

make_handles!(Data);

impl Handle {
    pub fn new_for_plain_text_file(path:fmc::Path, file_manager:file::Handle) -> Self {
        Handle::new_from_data(Data::new(path,file_manager))
    }

    pub fn new_for_module(module:module::Handle, file_manager:file::Handle) -> Self {
        let path     = module.location().to_path();
        let mut data = Data::new(path,file_manager);
        data.module = Some(module);
        Handle::new_from_data(data)
    }

    pub fn subscribe(&self) -> Subscriber<Notification> {
        self.with_borrowed(|data| data.notification_publisher.subscribe())
    }

    pub fn read_content(&self) -> impl Future<Output=Result<String,RpcError>> {
        let (file_manager,path) = self.with_borrowed(|data| (data.file_manager.clone(),data.file_path.clone()));
        file_manager.read(path)
    }
}