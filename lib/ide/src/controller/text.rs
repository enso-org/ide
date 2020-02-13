use crate::prelude::*;

use utils::make_handles;
use crate::controller::*;

use std::ops::Range;
use flo_stream::{Publisher, Subscriber};


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
#[derive(Clone,Debug)]
pub struct Data {
    /// A module handle in case the TextController is managing Luna module file.
    pub module: Option<module::Handle>,
    /// Sink where we put events to be consumed by the view.
    pub notifications_sender: Publisher<Notification>,
}

impl Data {
    fn new() -> Self {
        Data {
            module               : None,
            notifications_sender : Publisher::new(10),
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

make_handles!(Data);

impl Handle {
    pub fn new_for_plain_text_file() -> Self {
        Handle::new(Data::new())
    }

    pub fn new_for_module(module:module::Handle) {
        let mut data = Data::new();
        data.module = Some(module);
        Handle::new(data)
    }

    pub fn subscribe(&self) -> Subscriber<Notification> {
        self.with_borrowed_data(|data| data.notifications_sender.subscribe())
    }
}