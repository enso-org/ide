use crate::prelude::*;

use futures::task::LocalSpawnExt;
use std::collections::HashMap;

pub type FallibleResult<T> = std::result::Result<T,failure::Error>;

/// Macro defines `StrongHandle` and `WeakHandle` newtypes for handles storing
/// the type given in the argument.
///
/// This allows treating handles as separate types and fitting them with impl
/// methods of their own.
macro_rules! make_handles {
    ($data_type:ty) => {
        #[derive(Shrinkwrap)]
        #[derive(Clone,Debug)]
        pub struct StrongHandle(pub utils::cell::StrongHandle<$data_type>);

        impl StrongHandle {
            pub fn downgrade(&self) -> WeakHandle {
                WeakHandle(self.0.downgrade())
            }
            pub fn new(data:$data_type) -> StrongHandle {
                StrongHandle(utils::cell::StrongHandle::new(data))
            }
        }

        #[derive(Shrinkwrap)]
        #[derive(Clone,Debug)]
        pub struct WeakHandle(pub utils::cell::WeakHandle<$data_type>);

        impl WeakHandle {
            pub fn upgrade(&self) -> Option<StrongHandle> {
                self.0.upgrade().map(StrongHandle)
            }
        }
    };
}

mod project {
    //! Project controller.
    use super::*;
    use json_rpc::Transport;

    /// Create a new project controller.
    pub fn new(file_manager_transport:impl Transport + 'static) -> StrongHandle {
        let file_manager = file_manager_client::Client::new(file_manager_transport);
        let ret = Data {
            file_manager,
            modules: default(),
        };
        StrongHandle::new(ret)
    }

    /// Project controller's state.
    #[derive(Debug)]
    pub struct Data {
        file_manager : file_manager_client::Client,
        modules      : HashMap<module::Location, module::WeakHandle>,
    }

    impl Data {
        /// Returns handle to given module's controller if already present.
        pub fn lookup_module(&mut self, loc:&module::Location) -> Option<module::StrongHandle> {
            let weak = self.modules.get(loc)?;
            match weak.upgrade() {
                Some(strong) => Some(strong),
                None => {
                    self.modules.remove(loc);
                    None
                }
            }
        }

        /// Stores given module controller handle and returns it.
        ///
        /// Note: handle stored in the project controller is weak, so the caller
        /// remains responsible for managing the module's controller lifetime.
        pub fn insert_module(&mut self, data:module::Data) -> module::StrongHandle {
            let path   = data.loc.clone();
            let module = module::StrongHandle::new(data);
            let module_weak = module.downgrade();
            self.modules.insert(path,module_weak);
            module
        }

        /// Processes event from file manager.
        pub fn process_file_event(&mut self, _event:file_manager_client::Event) {
            // TODO push event to the appropriate module's input notification channel
            todo!()
        }

        /// Identifies module affected by given file manager's event.
        pub fn relevant_module(event:&file_manager_client::Event) -> Option<module::Location> {
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
            let handle        = self.clone();
            let processor_fut = fm_stream.for_each(move |event| {
                handle
                    .with(|data| data.process_file_event(event))
                    .map(|_| ()) // TODO handle error?
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

        pub async fn open_file_text(&self, _path:&std::path::Path) -> FallibleResult<text::StrongHandle> {
            // TODO [mwu] similar to the above
            todo!()
        }
    }

    /// boilerplate wrappers over stateful `Data` APIs
    impl StrongHandle {
        pub async fn lookup_module
        (&self, loc:module::Location)
         -> Option<module::StrongHandle> {
            self.with(move |data| data.lookup_module(&loc)).await
        }
        pub async fn insert_module
        (&self, module:module::Data)
         -> module::StrongHandle {
            self.with(|data| data.insert_module(module)).await
        }
        pub async fn call_fm<R>
        (&self, f:impl FnOnce(&mut file_manager_client::Client) -> R)
         -> R {
            self.with(|data| f(&mut data.file_manager)).await
        }
    }
}

/// Module controller.
mod module {
    use super::*;

    /// Structure uniquely identifying module location in the project.
    /// Mappable to filesystem path.
    #[derive(Clone,Debug,Eq,Hash,PartialEq)]
    pub struct Location(pub String);
    impl Location {
        pub fn to_path(&self) -> file_manager_client::Path {
            let result = format!("./{}.luna", self.0);
            file_manager_client::Path::new(result)
        }
    }

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
        pub fn fetch_text(&self) -> impl Future<Output = FallibleResult<String>> {
            let loc    = self.loc.clone();
            let parent = self.parent.clone();
            async move {
                parent.read_module(loc).await
            }
        }
    }

    make_handles!(Data);

    impl StrongHandle {
        pub fn fetch_text(&self) -> impl Future<Output = FallibleResult<String>> {
            self.with(|data| data.fetch_text()).flatten()
        }
    }
}

mod text {
    use super::*;

    /// A single set of edits. All edits use indices relative to the document's
    /// state from before any edits being applied.
    pub type Edits = Vec<Edit>;

    /// External context for this controller (underlying controller).
    #[derive(Clone,Debug)]
    pub enum Context {
        TextFromModule(module::StrongHandle),
        PlainTextFile(project::StrongHandle),
    }

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
                Context::PlainTextFile(project) =>
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


/////////////////////////////////////////////////

mod tests {
    use super::*;
    use crate::todo::executor::*;
    use json_rpc::test_util::transport::mock::*;

    //    #[test]
    async fn test_fn(transport:MockTransport) -> FallibleResult<()> {
        println!("start");

//        println!("run");
//        let project_conf = project::SetupConfiguration {
//            file_manager_endpoint: "ws://localhost:9001".into(),
//        };

//        let project = project::setup(&project_conf).await;
        let project = project::new(transport);
        let project_handle = project.downgrade();
        println!("has project");
        project.run_fm_events(global_spawner()).await?;
        println!("project loop started");

        let main_module_loc = module::Location("Luna".into());
        let module = project.open_module(&main_module_loc).await?;
        println!("module opened");

        let module_handle = module.downgrade();
        println!("done");


        Ok(())
    }

    #[test]
    fn test() {
        let mut transport = MockTransport::default();
        let mut executor = futures::executor::LocalPool::new();
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



//////////////////////////////////////////////////




