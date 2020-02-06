

use crate::prelude::*;
use std::collections::HashMap;
use futures::task::LocalSpawnExt;
use utils::cell;

pub type FallibleResult<T> = std::result::Result<T,failure::Error>;

/// Helper that generates wrapper tuple struct around Weak<RefCell<Data>>.
macro_rules! make_weak_handle {
    ($data_type:ty) => {
        #[derive(Shrinkwrap)]
        #[derive(Clone,Debug)]
        pub struct StrongHandle(pub utils::cell::StrongHandle<$data_type>);

        impl StrongHandle {
            pub fn downgrade(&self) -> WeakHandle {
                WeakHandle(Rc::downgrade(&self.0))
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

mod ide {
    use super::*;
    use json_rpc::test_util::transport::mock::MockTransport;

    /// Top-level function that creates a project controller.
    /// Automatically establishes WS connections with endpoints given in `conf`.
    pub async fn setup(conf:&SetupConfiguration) -> project::StrongHandle {
        let file_manager_transport = conf.connect_fm().await;
        project::new(file_manager_transport)
    }

    /// Configuration data necessary to setup the project controller.
    pub struct SetupConfiguration {
        /// URL of the websocket endpoint of the file manager server.
        pub file_manager_endpoint:String,
    }

    impl SetupConfiguration {
        /// Establishes connection with the remote file manager endpoint.
        pub async fn connect_fm(&self) -> MockTransport {
            // TODO [mwu] should not return mock transport but a real class
            //      implementing the websocket-based transport.
            todo!()
        }
    }
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
            let path   = data.path.clone();
            let module = StrongHandle::new(data);
            let module_weak = module.downgrade();
            self.modules.insert(path, module::WeakHandle{ data: module_weak });
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

    make_weak_handle!(Data);

//    #[derive(Shrinkwrap)]
//    #[derive(Clone,Debug)]
//    pub struct WeakHandle(pub utils::cell::WeakHandle<Data>);

    impl WeakHandle {
        /// Spawns the task that processes the file manager's events.
        pub async fn run_fm_events
        (&self, spawner:impl futures::task::LocalSpawn) -> FallibleResult<()> {
            let fm_stream     = self.call_fm(|fm| fm.events()).await?;
            let handle        = self.clone();
            let processor_fut = fm_stream.for_each(move |event| {
                handle
                    .with(|data| data.process_file_event(event))
                    .map(|_| ()) // TODO handle error?
            });

            spawner.spawn_local(processor_fut)?;
            Ok(())
        }

        /// Obtains the contents of the given module.
        pub async fn read_module
        (&self, loc:module::Location) ->  FallibleResult<String> {
            println!("will fetch contents of module {:?}", loc);
            let path = loc.to_path();
            let result_read_future = self.call_fm(|fm| fm.read(path)).await;
            // TODO how to map ok with async
            match result_read_future{
                Ok(read_future) =>
                    Ok(read_future.await?),
                Err(e) =>
                    Err(e.into()),
            }
        }

        /// Creates a new module controller.
        pub async fn create_module_controller
        (&self, loc:&module::Location) -> FallibleResult<module::StrongHandle> {
            let data = module::Data{
                path     : loc.clone(),
                contents : self.read_module(loc.clone()).await?,
                parent   : self.clone(),
            };
            let module = self.insert_module(data).await?;
            Ok(module)
        }
    }
    ////

    impl WeakHandle {
        /// Returns a module controller for given module location.
        ///
        /// Reuses existing controller if possible.
        /// Creates a new controller if needed.
        pub async fn open_module(&self, loc:&module::Location) -> FallibleResult<module::StrongHandle> {
            if let Some(existing_controller) = self.lookup_module(loc.clone()).await? {
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
    impl WeakHandle {
        pub async fn lookup_module
        (&self, loc:module::Location)
        -> cell::Result<Option<module::StrongHandle>> {
            self.with(move |data| data.lookup_module(&loc)).await
        }
        pub async fn insert_module
        (&self, module:module::Data)
        -> cell::Result<module::StrongHandle> {
            self.with(|data| data.insert_module(module)).await
        }
        pub async fn call_fm<R>
        (&self, f:impl FnOnce(&mut file_manager_client::Client) -> R)
        -> cell::Result<R> {
            self.with(|data| f(&mut data.file_manager)).await
        }
    }
}

/// Module controller.
mod module {
    use super::*;

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
        pub path     : Location,
        pub contents : String,
        pub parent   : project::WeakHandle,
    }

    impl Data {
        pub fn fetch_text(&self) -> impl Future<Output = FallibleResult<String>> {
            let loc    = self.path.clone();
            let parent = self.parent.clone();
            async move {
                parent.read_module(loc).await
            }
        }
    }

    make_weak_handle!(Data);

    impl WeakHandle {
        pub async fn fetch_text(&self) -> FallibleResult<String> {
            self.with(|data| data.fetch_text()).await?.await
        }
    }
}

mod text {
    use super::*;

    pub type StrongHandle = cell::StrongHandle<Data>;

    pub type Edits = Vec<Edit>;

    #[derive(Clone,Debug)]
    pub enum EventToView {
        /// File contents needs to be set to the following due to
        /// synchronization with external state.
        SetNewContent(String),
    }

    #[derive(Clone,Debug)]
    pub struct Edit {
        pub from     : usize,
        pub to       : usize,
        pub new_text : String,
    }

    #[derive(Clone,Debug)]
    pub struct Data {
        parent    : module::WeakHandle,
        tx_to_view: futures::channel::mpsc::UnboundedSender<EventToView>
    }

    impl Data {
        pub async fn file_externally_modified(&mut self) -> FallibleResult<()> {
            let new_text = self.parent.fetch_text().await?;
            let event = EventToView::SetNewContent(new_text);
            self.tx_to_view.unbounded_send(event)?;
            Ok(())
        }
    }

    #[derive(Shrinkwrap)]
    pub struct Handle(utils::cell::WeakHandle<Data>);
    impl Handle {
        pub fn apply_edits(edits: Vec<Edit>) {

        }
    }
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
        project_handle.run_fm_events(global_spawner()).await?;
        println!("project loop started");

        let main_module_loc = module::Location("Luna".into());
        let module = project_handle.open_module(&main_module_loc).await?;
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




