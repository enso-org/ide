//! Project controller.
//!
//! Responsible for owning any remote connection clients, and providing controllers for specific
//! files and modules. Expected to live as long as the project remains open in the IDE.

use crate::prelude::*;

use file_manager_client as fmc;
use json_rpc::Transport;
use parser::Parser;


// ===============================
// === Module Controller Cache ===
// ===============================


mod module_state_registry {
    use crate::prelude::*;

    use crate::controller::notification::Publisher;
    use crate::controller::module::State;
    use crate::controller::module::Location;

    use flo_stream::MessagePublisher;



    #[derive(Clone,Debug,Fail)]
    #[fail(display="Error while loading module")]
    struct LoadingError {}

    type LoadedNotification = Result<(), LoadingError>;

    #[derive(Debug,Clone)]
    enum Entry<Handle> {
        Loaded(Handle),
        Loading(Publisher<LoadedNotification>),
    }

    type StrongEntry = Entry<Rc<State>>;
    type WeakEntry   = Entry<Weak<State>>;

    impl WeakElement for WeakEntry {
        type Strong = StrongEntry;

        fn new(view: &Self::Strong) -> Self {
            match view {
                Entry::Loaded(handle)     => Entry::Loaded(Rc::downgrade(handle)),
                Entry::Loading(publisher) => Entry::Loading(publisher.clone()),
            }
        }

        fn view(&self) -> Option<Self::Strong> {
            match self {
                Entry::Loaded(handle)     => handle.upgrade().map(|h| Entry::Loaded(h)),
                Entry::Loading(publisher) => Some(Entry::Loading(publisher.clone()))
            }
        }
    }

    impl<Handle> Default for Entry<Handle> {
        fn default() -> Self {
            Self::Loading(default())
        }
    }

    impl<Handle:CloneRef> CloneRef for Entry<Handle> {}

    #[derive(Debug,Default)]
    pub struct ModuleRegistry {
        cache : RefCell<WeakValueHashMap<Location, WeakEntry>>
    }

    impl ModuleRegistry {

        pub async fn get_or_load<F>(&self, location:Location, loader:F) -> FallibleResult<Rc<State>>
        where F : Future<Output=FallibleResult<Rc<State>>> {
            match self.get(&location).await? {
                Some(state) => Ok(state),
                None        => self.load(location,loader).await
            }
        }

        async fn get(&self, location:&Location) -> Result<Option<Rc<State>>,LoadingError> {
            loop {
                let entry = self.cache.borrow_mut().get(&location);
                match entry {
                    Some(Entry::Loaded(state)) => { break Ok(Some(state)); },
                    Some(Entry::Loading(mut publisher)) => {
                        // Wait for loading to be finished.
                        publisher.subscribe().next().await.unwrap()?;
                    },
                    None => { break Ok(None); }
                }
            }

        }

        async fn load<F>(&self, loc:Location, loader:F) -> FallibleResult<Rc<State>>
        where F : Future<Output=FallibleResult<Rc<State>>> {
            let mut publisher = Publisher::default();
            self.cache.borrow_mut().insert(loc.clone(),Entry::Loading(publisher.clone()));

            let result = loader.await;
            match &result {
                Ok(state) => { self.cache.borrow_mut().insert(loc,Entry::Loaded(state.clone_ref())); },
                Err(_)    => { self.cache.borrow_mut().remove(&loc);                                 },
            }
            let message = match &result {
                Ok(_)  => Ok(()),
                Err(_) => Err(LoadingError{}),
            };
            publisher.publish(message);
            result
        }
    }
}



// ==========================
// === Project Controller ===
// ==========================

type ModuleLocation = controller::module::Location;


/// Project controller's state.
#[derive(Debug)]
pub struct Handle {
    /// File Manager Client.
    pub file_manager: fmc::Handle,
    /// Cache of module controllers.
    pub module_cache: Rc<module_state_registry::ModuleRegistry>,
    /// Parser handle.
    pub parser: Parser,
}

impl Handle {
    /// Create a new project controller.
    ///
    /// The remote connections should be already established.
    pub fn new(file_manager_transport:impl Transport + 'static) -> Self {
        Handle {
            file_manager : fmc::Handle::new(file_manager_transport),
            module_cache : default(),
            parser       : Parser::new_or_panic(),
        }
    }

    /// Creates a new project controller. Schedules all necessary execution with
    /// the global executor.
    pub fn new_running(file_manager_transport:impl Transport + 'static) -> Self {
        let ret = Self::new(file_manager_transport);
        crate::executor::global::spawn(ret.file_manager.runner());
        ret
    }

    /// Returns a text controller for given file path.
    ///
    /// It may be a controller for both modules and plain text files.
    pub async fn text_controller(&self, path:fmc::Path)
                                 -> FallibleResult<controller::text::Handle> {
        match ModuleLocation::from_path(&path) {
            Some(location) => {
                let module = self.module_controller(location).await?;
                Ok(controller::text::Handle::new_for_module(module))
            },
            None => {
                let fm = self.file_manager.clone_ref();
                Ok(controller::text::Handle::new_for_plain_text(path, fm))
            }
        }
    }

    /// Returns a module controller which have module opened from file.
    pub async fn module_controller
    (&self, location:ModuleLocation) -> FallibleResult<controller::module::Handle> {
        let state_loader = self.load_module(location.clone());
        let state        = self.module_cache.get_or_load(location.clone(),state_loader).await?;
        Ok(self.module_controller_with_state(location,state))
    }

    fn module_controller_with_state
    (&self, location:ModuleLocation, state:Rc<controller::module::State>)
    -> controller::module::Handle {
        let fm     = self.file_manager.clone_ref();
        let parser = self.parser.clone_ref();
        controller::module::Handle::new(location,state,fm,parser)
    }

    async fn load_module
    (&self, location:ModuleLocation) -> FallibleResult<Rc<controller::module::State>> {
        let state  = Rc::<controller::module::State>::default();
        let module = self.module_controller_with_state(location,state.clone_ref());
        module.load_file().await.map(move |()| state)
    }
}



#[cfg(test)]
mod test {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::transport::test_utils::TestWithMockedTransport;

    use file_manager_client::Path;
    use json_rpc::test_util::transport::mock::MockTransport;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;


    wasm_bindgen_test_configure!(run_in_browser);



    #[wasm_bindgen_test]
    fn obtain_module_controller() {
        let transport = MockTransport::new();
        let mut test  = TestWithMockedTransport::set_up(&transport);
        test.run_test(async move {
            let project_ctrl = controller::project::Handle::new_running(transport);
            let location     = controller::module::Location::new("TestLocation");
            let another_loc  = controller::module::Location::new("TestLocation2");

            let module_ctrl         = project_ctrl.module_controller(location.clone()).await.unwrap();
            let same_module_ctrl    = project_ctrl.module_controller(location.clone()).await.unwrap();
            let another_module_ctrl = project_ctrl.module_controller(another_loc.clone()).await.unwrap();

            assert_eq!(location   , module_ctrl        .location);
            assert_eq!(another_loc, another_module_ctrl.location);
            assert!(Rc::ptr_eq(&module_ctrl.module, &same_module_ctrl.module));
        });

        test.when_stalled_send_response("2 + 2");
        test.when_stalled_send_response("3+3");
    }

    #[wasm_bindgen_test]
    fn obtain_plain_text_controller() {
        let transport       = MockTransport::new();
        TestWithLocalPoolExecutor::set_up().run_test(async move {
            let project_ctrl        = controller::project::Handle::new_running(transport);
            let path                = Path::new("TestPath");
            let another_path        = Path::new("TestPath2");

            let text_ctrl    = project_ctrl.text_controller(path.clone()).await.unwrap();
            let another_ctrl = project_ctrl.text_controller(another_path.clone()).await.unwrap();

            assert!(project_ctrl.file_manager.identity_equals(&text_ctrl   .file_manager()));
            assert!(project_ctrl.file_manager.identity_equals(&another_ctrl.file_manager()));
            assert_eq!(path        , text_ctrl   .file_path()  );
            assert_eq!(another_path, another_ctrl.file_path()  );
        });
    }

    #[wasm_bindgen_test]
    fn obtain_text_controller_for_module() {
        let transport       = MockTransport::new();
        let mut test        = TestWithMockedTransport::set_up(&transport);
        test.run_test(async move {
            let project_ctrl = controller::project::Handle::new_running(transport);
            let path         = controller::module::Location::new("test").to_path();
            let text_ctrl    = project_ctrl.text_controller(path.clone_ref()).await.unwrap();
            let content      = text_ctrl.read_content().await.unwrap();
            assert_eq!("2 + 2", content.as_str());
        });
        test.when_stalled_send_response("2 + 2");
    }
}
