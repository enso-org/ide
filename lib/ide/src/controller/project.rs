//! Project controller.
//!
//! Responsible for owning any remote connection clients. Expected to live
//! as long as the project remains open in the IDE.

use crate::prelude::*;

use crate::controller::*;

use utils::make_handles;
use json_rpc::Transport;
use weak_table::WeakValueHashMap;
use weak_table::weak_value_hash_map::Entry::Occupied;
use weak_table::weak_value_hash_map::Entry::Vacant;
use flo_stream::Publisher;
use file_manager_client as fmc;
use file_manager_client::Notification;
use file_manager_client::FilesystemEvent;
use futures::SinkExt;


/// Project controller's state.
#[derive(Debug)]
pub struct Data {
    /// File Manager Client.
    file_manager: file::Handle,
    /// Cache of module controllers.
    module_cache: WeakValueHashMap<module::Location, module::WeakHandle>,
    /// Cache of text controllers.
    text_cache: WeakValueHashMap<file_manager_client::Path,text::WeakHandle>,
}

make_handles!(Data);

impl Handle {
    /// Create a new project controller.
    ///
    /// The remote connections should be already established.
    pub fn new(file_manager_transport:impl Transport + 'static) -> Self {
        let data = Data {
            file_manager           : file::Handle::new(file_manager_transport),
            module_cache           : default(),
            text_cache             : default(),
        };
        Handle::new_from_data(data)
    }

    /// Returns a module controller for given module location.
    ///
    /// Reuses existing controller if possible.
    /// Creates a new controller if needed.
    pub fn open_module(&self, loc:module::Location) -> FallibleResult<module::Handle> {
        self.with_borrowed(|data| {
            match data.module_cache.entry(loc.clone()) {
                Occupied(entry) => Ok(entry.get().clone()),
                Vacant(entry)   => Ok(entry.insert(module::Handle::new(loc))),
            }
        })
    }

    pub fn open_text_file(&self, path:file_manager_client::Path) -> text::Handle {
        self.with_borrowed(|data| {
            let fm = data.file_manager.clone();
            match data.text_cache.entry(path.clone()) {
                Occupied(entry) => entry.get().clone(),
                Vacant(entry)   => entry.insert(text::Handle::new_for_plain_text_file(path,fm)),
            }
        })
    }
}

impl Data {
    /// Obtains a handle to a module controller interested in this
    /// filesystem event.
    fn relevant_module
    (&mut self, event:&file_manager_client::Event) -> Option<module::Handle> {
        let location = Self::relevant_location(event)?;
        self.module_cache.get(&location)
    }

    /// Identifies module affected by given file manager's event.
    fn relevant_location(event:&file_manager_client::Event) -> Option<module::Location> {
        use file_manager_client::Event;
        use file_manager_client::Notification::FilesystemEvent;
        match event {
            Event::Closed          => None,
            Event::Error(_)        => None,
            Event::Notification(n) => match n {
                FilesystemEvent(e) => Some(module::Location(e.path.0.clone()))
            }
        }
    }
}