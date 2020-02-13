use crate::prelude::*;

use utils::make_handles;
use crate::controller::*;

use std::ops::Range;
use flo_stream::{Publisher, Subscriber};
use flo_stream::MessagePublisher;
use failure::_core::fmt::{Formatter, Error};


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
    /// A module handle in case the TextController is managing Luna module file.
    pub module: Option<module::Handle>,
    /// Sink where we put events to be consumed by the view.
    pub notification_publisher: Publisher<Notification>,
}

impl Data {
    fn new() -> Self {
        Data {
            module                 : None,
            notification_publisher : Publisher::new(10),
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Text Controller for module {:?}", self.module)
    }
}

make_handles!(Data);

impl Handle {
    pub fn new_for_plain_text_file() -> Self {
        Handle::new(Data::new())
    }

    pub fn new_for_module(module:module::Handle) -> Self {
        let mut data = Data::new();
        data.module = Some(module);
        Handle::new(data)
    }

    pub fn subscribe(&self) -> Subscriber<Notification> {
        self.with_borrowed(|data| data.notification_publisher.subscribe())
    }
}