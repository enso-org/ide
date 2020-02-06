

use crate::prelude::*;
use std::collections::HashMap;
use futures::task::{SpawnExt, LocalSpawnExt};

use utils::handle::*;

mod project {
    use super::*;
    use crate::todo::executor::*;

    pub async fn setup(conf:&SetupConfiguration) -> StrongHandle<project::Data> {
        let file_manager = crate::setup_file_manager(&conf.file_manager_endpoint).await;
        let ret = Data{
            file_manager,
            modules: default(),
        };
        let ret  = strong(ret);
        let weak = WeakHandle(Rc::downgrade(&ret));
        let processor_fut = weak.event_processor().unwrap();
        spawn_task(processor_fut);
        ret
    }

    pub struct SetupConfiguration {
        pub file_manager_endpoint:String,
    }

    pub struct Data {
        file_manager: file_manager_client::Client,
        modules     : HashMap<module::Location, module::WeakHandle>,
    }

    impl Data {
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
        pub fn insert_module(&mut self, data:module::Data) -> module::StrongHandle {
            let path = data.path.clone();
            let module = strong(data);
            self.modules.insert(path, module::WeakHandle{data: Rc::downgrade(&module)});
            module
        }

        pub fn process_file_event(&mut self, event:file_manager_client::Event) {
            if let Some(module_loc) = Self::relevant_module(&event) {
                if let Some(module) = self.modules.get(&module_loc) {
                    module.with_data(|data| data.handle_event(module::Event::FileChanged));
                }
            }
        }
        ////

        pub fn relevant_module(event:&file_manager_client::Event) -> Option<module::Location> {
            use file_manager_client::Event;
            use file_manager_client::Notification::*;
            match event {
                Event::Closed => None,
                Event::Error(e) => None,
                Event::Notification(n) => match n {
                    FilesystemEvent(e) => Some(module::Location(e.path.0.clone()))
                }
            }
        }
    }

    #[derive(Clone)]
    pub struct WeakHandle(pub Weak<RefCell<Data>>);

    impl WeakHandle {
        pub async fn read_module(&self, loc:&module::Location) ->  FallibleResult<String> {
            let path = loc.to_path();
            Ok(self.call_fm(|fm| fm.read(path))?.await?)
        }

        pub async fn create_module_controller(&self, loc:&module::Location) -> FallibleResult<module::StrongHandle> {
            let path = loc.to_path();
            let contents = self.read_module(&loc).await?;
            let data = module::Data{
                path:loc.clone(),
                contents,
                parent: self.clone(),
            };
            let module = self.insert_module(data)?;
            Ok(module)
        }

        pub fn event_processor(&self) -> FallibleResult<impl Future<Output = ()>> {
            let events_stream = self.call_fm(|fm| fm.events())?;

            let mut weak_handle = self.clone();
            Ok(events_stream.for_each(move |event| {
                let result = weak_handle.clone().with_data(|data| data.process_file_event(event));
                futures::future::ready(())
            }))
        }
    }
    ////

    impl WeakHandle {
        pub async fn open_module(&self, loc:&module::Location) -> FallibleResult<module::StrongHandle> {
            if let Some(existing_controller) = self.lookup_module(&loc)? {
                Ok(existing_controller)
            } else if let new_controller = self.create_module_controller(&loc).await? {
                Ok(new_controller)
            } else {
                panic!("Failed to create a controller for {}",loc.0);
            }
        }
    }

    /////////////////////////////////
    // BOILERPLATE TO BE GENERATED //
    /////////////////////////////////
    impl IsWeakHandle for WeakHandle {
        type Data = Data;
        fn weak_handle(&self) -> Weak<RefCell<Self::Data>> {
            self.0.clone()
        }
    }

    /// boilerplate wrappers over internal APIs
    impl WeakHandle {
        pub fn lookup_module(&self, loc:&module::Location) -> Result<Option<module::StrongHandle>> {
            self.with_data(move |data| data.lookup_module(loc))
        }
        pub fn insert_module(&self, module:module::Data) -> Result<module::StrongHandle> {
            self.with_data(move |data| data.insert_module(module))
        }
        pub fn call_fm<R>(&self, f:impl FnOnce(&mut file_manager_client::Client) -> R) -> Result<R> {
            self.with_data(|data| f(&mut data.file_manager))
        }
    }
}

mod module {
    use super::*;

    pub enum Event {
        FileChanged,
    }

    #[derive(Clone,Eq,Hash,PartialEq)]
    pub struct Location(pub String);
    impl Location {
        pub fn to_path(&self) -> file_manager_client::Path {
            let result = format!("./{}.luna", self.0);
            file_manager_client::Path::new(result)
        }
    }

    pub struct Data {
        pub path: Location,
        pub contents: String,
        pub parent: project::WeakHandle,
    }

    impl Data {
        pub async fn fetch_contents(&self) -> FallibleResult<String> {
            self.parent.read_module(&self.path).await
        }

        pub fn handle_event(&mut self, event:Event) {

        }
    }

    pub type StrongHandle = Rc<RefCell<Data>>;

    #[derive(Clone)]
    pub struct WeakHandle {
        pub data: Weak<RefCell<Data>>,
    }

    impl WeakHandle {
        fn from_weak(data:Weak<RefCell<Data>>) -> Self {
            WeakHandle { data }
        }
//        pub fn path(&self) -> FailableResult<Location> {
//            Ok(self.with_data(|data| data.path.clone())?)
//        }
    }

    impl IsWeakHandle for WeakHandle {
        type Data = Data;
        fn weak_handle(&self) -> Weak<RefCell<Self::Data>> {
            self.data.clone()
        }
    }
}

mod text {
    use super::*;

    pub enum Notification {
        DocumentExternallyChanged{ new_contents:String }
    }

    pub struct Edit {
        pub from     : usize,
        pub to       : usize,
        pub new_text : String,
    }

    impl Edit {
    }

    struct Data {
        parent: module::WeakHandle,
    }

    struct Handle;
    impl Handle {
        pub fn apply_edits(edits: Vec<Edit>) {}
//        pub fn notifications()
    }
}

/////////////////////////////////////////////////

mod tests {
    use super::*;
    use crate::todo::executor::*;

    //    #[test]
    async fn test_fn() -> FallibleResult<()> {
        println!("run");
        let project_conf = project::SetupConfiguration {
            file_manager_endpoint: "ws://localhost:9001".into(),
        };

        let project = project::setup(&project_conf).await;
        let project_handle = project::WeakHandle(Rc::downgrade(&project));
        let main_module_loc = module::Location("Luna".into());

        let module = project_handle.open_module(&main_module_loc).await?;
        let module_handle = module::WeakHandle{data : (Rc::downgrade(&module)) };


        Ok(())
    }

    #[test]
    fn test() {
        let mut executor = futures::executor::LocalPool::new();
        set_global_spawner(executor.spawner());
        spawn_task(test_fn().then(|r| {
            r.expect("not ok");
            futures::future::ready(())
        }));
        executor.run_until_stalled();

    }
}

//////////////////////////////////////////////////




