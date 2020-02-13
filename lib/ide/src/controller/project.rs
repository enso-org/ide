//! Project controller.
//!
//! Responsible for owning any remote connection clients. Expected to live
//! as long as the project remains open in the IDE.

use crate::prelude::*;

use crate::controller::*;

use utils::make_handles;
use json_rpc::Transport;

/// Create a new project controller.
///
/// The remote connections should be already established.
pub fn new(file_manager_transport:impl Transport + 'static) -> Handle {
    let data = Data {
        file_manager : file_manager_client::Client::new(file_manager_transport),
        module_cache : default(),
        text_cache   : default(),
    };
    Handle::new(data)
}

/// Project controller's state.
#[derive(Debug)]
pub struct Data {
    /// File Manager Client.
    file_manager: file_manager_client::Client,
    /// Cache of module controllers.
    module_cache: HashMap<module::Location, module::WeakHandle>,
    /// Cache of text controllers.
    text_cache: HashMap<file_manager_client::Path,text::WeakHandle>,
}

make_handles!(Data);

impl Handle {
    /// Spawns the task that processes the file manager's events.
    ///
    /// Note that File Manager requires manually calling `process_events`
    /// method to yield any events through its streams / futures.
    /// (without such call, this processor will just wait indefinitely)
    pub async fn run_fm_events
    (&self, spawner:impl futures::task::LocalSpawn) -> FallibleResult<()> {
        let fm_stream     = self.call_fm(|fm| fm.events()).await;
        let processor_fut = fm_stream.for_each(move |_event| async {
            // TODO [mwu] dispatch the notification to the appropriate module
            todo!()
        });

        Ok(spawner.spawn_local(processor_fut)?)
    }

    /// Returns a module controller for given module location.
    ///
    /// Reuses existing controller if possible.
    /// Creates a new controller if needed.
    pub fn open_module(&self, loc:&module::Location) -> FallibleResult<module::Handle> {
        self.with_borrowed(|data| {
            if let Some(existing_controller) = data.lookup_module(loc.clone()) {
                Ok(existing_controller)
            } else {
                data.create_module_controller(&loc)
            }
        })
    }

    /// Retuns a text controller for given file path. It may designate
    /// either the Luna source file or other file belonging to the project.
    ///
    /// File should be an existing, correct UTF-8 encoded file.
    pub async fn open_file_text(&self, _path:&std::path::Path) -> FallibleResult<text::Handle> {
        // TODO [mwu] similar to the above (will need to add a second map
        //      to keep handles or perhaps abstract the handles map to a
        //      reusable structure?
        //      Also might need to prepare a module first.
        todo!()
    }
}

impl Data {
    /// Returns handle to given module's controller if already present.
    fn lookup_module(&mut self, loc:&module::Location) -> Option<module::Handle> {
        let weak   = self.module_cache.get(loc)?;
        let handle = weak.upgrade();
        if handle.is_none() {
            self.module_cache.remove(loc);
        }
        handle
    }

    /// Stores given module controller handle and returns it.
    ///
    /// Note: handle stored in the project controller is weak, so the caller
    /// remains responsible for managing the module's controller lifetime
    /// (this is why the strong handle is returned).
    ///
    /// If there was already another module controller present in the cache,
    /// it will be overwritten. Typically caller should use `lookup_module`
    /// first.
    fn insert_module(&mut self, data:module::Data) -> module::Handle {
        let path        = data.loc.clone();
        let module      = module::Handle::new(data);
        let module_weak = module.downgrade();
        self.module_cache.insert(path, module_weak);
        module
    }

    /// Obtains a handle to a module controller interested in this
    /// filesystem event.
    fn relevant_module
    (&mut self, event:&file_manager_client::Event) -> Option<module::Handle> {
        let location = Self::relevant_location(event)?;
        self.lookup_module(&location)
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

    /// Creates a new module controller.
    fn create_module_controller(&mut self, loc:&module::Location)
    -> FallibleResult<module::Handle> {
        let module_data = module::Data{
            loc : loc.clone(),
        };
        let module = self.insert_module(module_data);
        Ok(module)
    }
}