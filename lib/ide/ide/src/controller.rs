//! This module contains all the controllers. They cover everything that is
//! between clients of remote services (like language server and file manager)
//! and views.
//!
//! The controllers create a tree-like structure, with project controller being
//! a root, then module controllers below, then graph/text controller and so on.
//!
//! As a general rule, while the "upper" (i.e. closer to root) nodes may keep
//! handles to the "lower" nodes (e.g. to allow their reuse), they should never
//! manage their lifetime.
//!
//! Primarily views are considered owners of their respective controllers.
//! Additionally, controllers are allowed to keep strong handle "upwards".
//!
//! Controllers store their handles using `utils::cell` handle types to ensure
//! that mutable state is safely accessed.

use crate::prelude::*;

/// General-purpose `Result` supporting any `Error`-compatible failures.
pub type FallibleResult<T> = Result<T,failure::Error>;

/// Macro defines `StrongHandle` and `WeakHandle` newtypes for handles storing
/// the type given in the argument.
///
/// This allows treating handles as separate types and fitting them with impl
/// methods of their own. While not necessary, such implementation may allow
/// hiding from user gritty details of `with` usage behind nice, easy API.
macro_rules! make_handles {
    ($data_type:ty) => {
        /// newtype wrapper over StrongHandle.
        #[derive(Shrinkwrap)]
        #[derive(Clone,Debug)]
        pub struct StrongHandle(pub utils::cell::StrongHandle<$data_type>);

        impl StrongHandle {
            /// Obtain a WeakHandle to this data.
            pub fn downgrade(&self) -> WeakHandle {
                WeakHandle(self.0.downgrade())
            }
            /// Create a new StrongHandle that will wrap given data.
            pub fn new(data:$data_type) -> StrongHandle {
                StrongHandle(utils::cell::StrongHandle::new(data))
            }
        }

        /// newtype wrapper over WeakHandle.
        #[derive(Shrinkwrap)]
        #[derive(Clone,Debug)]
        pub struct WeakHandle(pub utils::cell::WeakHandle<$data_type>);

        impl WeakHandle {
            /// Obtain a StrongHandle to this data.
            pub fn upgrade(&self) -> Option<StrongHandle> {
                self.0.upgrade().map(StrongHandle)
            }
        }
    };
}



// =======================
// === Text controller ===
// =======================

pub mod project {
    //! Project controller.
    ///
    /// Responsible for owning any remote connection clients. Expected to live
    /// as long as the project remains open in the IDE.
    use super::*;
    use json_rpc::Transport;

    /// Create a new project controller.
    ///
    /// The remote connections should be already established.
    pub fn new(file_manager_transport:impl Transport + 'static) -> StrongHandle {
        let file_manager = file_manager_client::Client::new(file_manager_transport);
        let ret = Data {
            file_manager,
            module_cache: default(),
        };
        StrongHandle::new(ret)
    }

    /// Project controller's state.
    #[derive(Debug)]
    pub struct Data {
        /// File Manager Client.
        file_manager : file_manager_client::Client,
        /// Cache of module controllers. As the weak is handle (we don't manage
        /// their lifetime), dead ones might still be included.
        module_cache : HashMap<module::Location, module::WeakHandle>,
    }

    impl Data {
        /// Returns handle to given module's controller if already present.
        pub fn lookup_module(&mut self, loc:&module::Location) -> Option<module::StrongHandle> {
            let weak = self.module_cache.get(loc)?;
            match weak.upgrade() {
                Some(strong) => Some(strong),
                None => {
                    self.module_cache.remove(loc);
                    None
                }
            }
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
        pub fn insert_module(&mut self, data:module::Data) -> module::StrongHandle {
            let path        = data.loc.clone();
            let module      = module::StrongHandle::new(data);
            let module_weak = module.downgrade();
            self.module_cache.insert(path, module_weak);
            module
        }

        /// Obtains a handle to a module controller interested in this
        /// filesystem event.
        pub fn relevant_module
        (&mut self, event:&file_manager_client::Event) -> Option<module::StrongHandle> {
            let location = Self::relevant_location(event)?;
            self.lookup_module(&location)
        }

        /// Identifies module affected by given file manager's event.
        pub fn relevant_location(event:&file_manager_client::Event) -> Option<module::Location> {
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

    make_handles!(Data);

    impl StrongHandle {
        /// Obtains the contents of the given module.
        pub async fn read_module
        (&self, loc:module::Location) ->  FallibleResult<String> {
            println!("will fetch contents of module {:?}", loc);
            let path = loc.to_path();
            let read_future = self.call_fm(|fm| fm.read(path)).await;
            Ok(read_future.await?)
        }

        /// Creates a new module controller.
        pub async fn create_module_controller
        (&self, loc:&module::Location) -> FallibleResult<module::StrongHandle> {
            let data = module::Data{
                loc      : loc.clone(),
                contents : self.read_module(loc.clone()).await?,
                parent   : self.clone(),
            };
            let module = self.insert_module(data).await;
            Ok(module)
        }

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
        pub async fn open_module(&self, loc:&module::Location) -> FallibleResult<module::StrongHandle> {
            if let Some(existing_controller) = self.lookup_module(loc.clone()).await {
                Ok(existing_controller)
            } else {
                Ok(self.create_module_controller(&loc).await?)
            }
        }

        /// Retuns a text controller for given file path. It may designate
        /// either the Luna source file or other file belonging to the project.
        ///
        /// File should be an existing, correct UTF-8 encoded file.
        pub async fn open_file_text(&self, _path:&std::path::Path) -> FallibleResult<text::StrongHandle> {
            // TODO [mwu] similar to the above (will need to add a second map
            //      to keep handles or perhaps abstract the handles map to a
            //      reusable structure?
            //      Also might need to prepare a module first.
            todo!()
        }
    }

    /// Boilerplate wrappers over stateful `Data` APIs. Refer to the `Data`
    /// methods for documentation.
    impl StrongHandle {
        #[allow(missing_docs)]
        pub async fn lookup_module
        (&self, loc:module::Location)
         -> Option<module::StrongHandle> {
            self.with(move |data| data.lookup_module(&loc)).await
        }
        #[allow(missing_docs)]
        pub async fn insert_module
        (&self, module:module::Data)
         -> module::StrongHandle {
            self.with(|data| data.insert_module(module)).await
        }
        #[allow(missing_docs)]
        pub async fn call_fm<R>
        (&self, f:impl FnOnce(&mut file_manager_client::Client) -> R)
         -> R {
            self.with(|data| f(&mut data.file_manager)).await
        }
    }
}



// =======================
// === Text controller ===
// =======================

/// Module controller.
pub mod module {
    use super::*;

    /// Structure uniquely identifying module location in the project.
    /// Mappable to filesystem path.
    #[derive(Clone,Debug,Eq,Hash,PartialEq)]
    pub struct Location(pub String);
    impl Location {
        /// Obtains path (within a project context) to the file with this module.
        pub fn to_path(&self) -> file_manager_client::Path {
            // TODO [mwu] Extremely provisional. When multiple files support is
            //            added, needs to be fixed, if not earlier.
            let result = format!("./{}.luna", self.0);
            file_manager_client::Path::new(result)
        }
    }

    /// State data of the module controller.
    #[derive(Clone,Debug)]
    pub struct Data {
        /// This module's location.
        pub loc      : Location,
        /// Contents of the module file.
        pub contents : String,
        /// Handle to the project.
        pub parent   : project::StrongHandle,
    }

    impl Data {
        /// Fetches the Luna code for this module using remote File Manager.
        pub fn fetch_text(&self) -> impl Future<Output = FallibleResult<String>> {
            let loc    = self.loc.clone();
            let parent = self.parent.clone();
            // TODO [mwu] When metadata support is added, they will need to be
            //            stripped together with idmap from the source code.
            async move {
                parent.read_module(loc).await
            }
        }
    }

    make_handles!(Data);

    impl StrongHandle {
        /// Fetches the Luna code for this module using remote File Manager.
        pub fn fetch_text(&self) -> impl Future<Output = FallibleResult<String>> {
            self.with(|data| data.fetch_text()).flatten()
        }

        /// Receives a notification call when file with this module has been
        /// modified by a third-party tool (like non-IDE text editor).
        pub async fn file_externally_modified(&self) {
            // TODO: notify underlying text/graph controllers about the changes
            todo!()
        }
    }
}



// =======================
// === Text controller ===
// =======================

/// Text controller.
pub mod text {
    use super::*;

    /// A single set of edits. All edits use indices relative to the document's
    /// state from before any edits being applied.
    pub type Edits = Vec<Edit>;

    /// External context for this controller (underlying controller).
    #[derive(Clone,Debug)]
    pub enum Context {
        /// Controller for the Luna module that we are displaying.
        TextFromModule(module::StrongHandle),
        /// Controller for the project with the non-Luna file we are displaying.
        PlainTextFile(project::StrongHandle),
    }

    /// Events sent from text controller to its view.
    #[derive(Clone,Debug)]
    pub enum EventToView {
        /// File contents needs to be set to the following due to
        /// synchronization with external state.
        SetNewContent(String),
    }

    /// Edit action on the text document that replaces text on given span with
    /// a new one.
    #[derive(Clone,Debug)]
    pub struct Edit {
        /// Replaced range begin.
        pub from     : usize,
        /// Replaced range end (after last replaced character).
        /// If same value as `from` this is insert operation.
        pub to       : usize,
        /// Text to be placed. May be empty to erase portion of text.
        pub new_text : String,
    }

    /// Data stored by the text controller.
    #[derive(Clone,Debug)]
    pub struct Data {
        /// Context, i.e. entity that we can query for externally-synchronized
        /// text content.
        pub context    : Context,
        /// Sink where we put events to be consumed by the view.
        pub tx_to_view : futures::channel::mpsc::UnboundedSender<EventToView>
    }

    impl Data {
        /// Method called by the context when the file was externally modified.
        /// (externally, as in not by the view we are connected with)
        pub async fn file_externally_modified(&mut self) -> FallibleResult<()> {
            let new_text = match &self.context {
                Context::TextFromModule(module) =>
                    module.fetch_text().await?,
                Context::PlainTextFile(_project) =>
                    // TODO [mwu] fetch the text directly through project
                    //      manager or the file manager (whatever is deemed to
                    //      be more appropriate as a context provider here)
                    todo!(),
            };
            let event = EventToView::SetNewContent(new_text);
            self.tx_to_view.unbounded_send(event)?;
            Ok(())
        }

        /// View can at any point request setting up the channel, in such case
        /// any previous channel is abandoned and subsequent event will be
        /// obtainable through the returned receiver.
        pub fn setup_stream_to_view(&mut self) -> futures::channel::mpsc::UnboundedReceiver<EventToView> {
            let (tx,rx) = futures::channel::mpsc::unbounded();
            self.tx_to_view = tx;
            rx
        }
    }

    make_handles!(Data);
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::executor::*;
    use json_rpc::test_util::transport::mock::*;

    async fn test_fn(transport:MockTransport) -> FallibleResult<()> {
        println!("start");
        let project = project::new(transport);
        println!("has project");
        project.run_fm_events(current_spawner()).await?;
        println!("project loop started");
        let main_module_loc = module::Location("Luna".into());
        let _module = project.open_module(&main_module_loc).await?;
        println!("module opened");
        println!("done");
        Ok(())
    }

    #[test]
    fn test() {
        let mut transport = MockTransport::default();
        let mut executor  = futures::executor::LocalPool::new();
        set_global_spawner(executor.spawner());
        spawn_task(test_fn(transport.clone()).then(|r| {
            r.expect("not ok");
            futures::future::ready(())
        }));
        executor.run_until_stalled();
        transport.mock_peer_message(json_rpc::messages::Message::new_success(json_rpc::messages::Id(0), "foo"));
        executor.run_until_stalled();
    }
}
