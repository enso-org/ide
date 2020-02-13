use crate::prelude::*;

use utils::make_handles;
use crate::controller::*;

use std::ops::Range;
use flo_stream::{Publisher, Subscriber};
use flo_stream::MessagePublisher;
use failure::_core::fmt::{Formatter, Error};
use file_manager_client as fmc;
use json_rpc::Transport;
use json_rpc::error::RpcError;

make_handles!(fmc::Client);

impl Handle {
    /// Create a new project controller.
    ///
    /// The remote connections should be already established.
    pub fn new(file_manager_transport:impl Transport + 'static) -> Self {
        Handle::new_from_data(fmc::Client::new(file_manager_transport))
    }

    pub fn read(&self, path:fmc::Path) -> impl Future<Output=Result<String,RpcError>> {
        self.with_borrowed(|client| client.read(path))
    }
}