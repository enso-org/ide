//! Project controller.
//!
//! Responsible for owning any remote connection clients, and providing controllers for specific
//! files and modules. Expected to live as long as the project remains open in the IDE.

use crate::prelude::*;

use enso_protocol::file_manager as fmc;
use fmc::API;
use json_rpc::Transport;
use parser::Parser;



// ==========================
// === Project Controller ===
// ==========================



/// Project controller's state.
#[derive(Debug)]
pub struct Handle {
    /// File Manager Client.
    pub file_manager : Rc<fmc::Client>,
    /// The project's root ID in file manager.
    pub root_id : uuid::Uuid,
    /// Cache of module controllers.
    pub module_registry : Rc<model::module::registry::Registry>,
    /// Parser handle.
    pub parser : Parser,
}

impl Handle {
    /// Create a new project controller.
    ///
    /// The remote connection should be already established.
    pub async fn new(file_manager_transport:impl Transport + 'static) -> Self {
        let file_manager    = Rc::new(fmc::Client::new(file_manager_transport));
        let module_registry = default();
        let parser          = Parser::new_or_panic();
        let root_id         = default();
        Handle {root_id,file_manager,module_registry,parser}
    }

    /// Creates a new project controller. Schedules all necessary execution with
    /// the global executor.
    pub async fn new_running(file_manager_transport:impl Transport + 'static) -> Self {
        let mut ret = Self::new(file_manager_transport).await;
        println!("Spawning runner.");
        crate::executor::global::spawn(ret.file_manager.runner());
        ret.initialize_protocol_connection().await;
        ret
    }

    /// Initialize the connection used to send the textual protocol messages. This initialisation
    /// is important such that the client identifier can be correlated between the textual and data
    /// connections.
    pub async fn initialize_protocol_connection(&mut self) {
        //FIXME[dg]: We need to make use of a proper client ID. I am still not sure where it
        // should come from so clarification is needed.
        let client_id = default();
        println!("Initializing protocol connection.");
        let response  = self.file_manager.init_protocol_connection(client_id).await;
        println!("Initialized protocol connection.");
        let response  = response.expect("Couldn't get project content roots.");
        //FIXME[dg]: We will make use of the first available `root_id`s, but we should expand this
        // logic to make use of the all available `root_id`s.
        self.root_id  = response.content_roots[0];
    }

    /// Returns a text controller for given file path.
    ///
    /// It may be a controller for both modules and plain text files.
    pub async fn text_controller(&self, path:fmc::Path) -> FallibleResult<controller::Text> {
        let module = self.module_controller(path).await?;
        Ok(controller::Text::new_for_module(module))
    }

    /// Returns a module controller which have module opened from file.
    pub async fn module_controller
    (&self, path:fmc::Path) -> FallibleResult<controller::Module> {
        let model_loader = self.load_module(path.clone());
        let model        = self.module_registry.get_or_load(path.clone(), model_loader).await?;
        Ok(self.module_controller_with_model(path,model))
    }

    fn module_controller_with_model
    (&self, path:fmc::Path, model:Rc<model::Module>) -> controller::Module {
        let fm     = self.file_manager.clone();
        let parser = self.parser.clone_ref();
        controller::Module::new(path, model, fm, parser)
    }

    async fn load_module(&self, path:fmc::Path) -> FallibleResult<Rc<model::Module>> {
        let model  = Rc::<model::Module>::default();
        let module = self.module_controller_with_model(path, model.clone_ref());
        module.load_file().await.map(move |()| model)
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
            let project     = controller::Project::new_running(transport).await;
            let location    = ModuleLocation::new("TestLocation");
            let another_loc = ModuleLocation::new("TestLocation2");

            let module         = project.module_controller(location.clone()).await.unwrap();
            let same_module    = project.module_controller(location.clone()).await.unwrap();
            let another_module = project.module_controller(another_loc.clone()).await.unwrap();

            assert_eq!(location,    module.location);
            assert_eq!(another_loc, another_module.location);
            assert!(Rc::ptr_eq(&module.model, &same_module.model));
        });

        test.when_stalled_send_response("2 + 2");
        test.when_stalled_send_response("3+3");
    }

    #[wasm_bindgen_test]
    fn obtain_plain_text_controller() {
        let transport       = MockTransport::new();
        TestWithLocalPoolExecutor::set_up().run_task(async move {
            let project_ctrl        = controller::Project::new_running(transport).await;
            let path                = Path::new("TestPath");
            let another_path        = Path::new("TestPath2");

            let text_ctrl    = project_ctrl.text_controller(path.clone()).await.unwrap();
            let another_ctrl = project_ctrl.text_controller(another_path.clone()).await.unwrap();

            assert!(project_ctrl.file_manager.identity_equals(&text_ctrl   .file_manager()));
            assert!(project_ctrl.file_manager.identity_equals(&another_ctrl.file_manager()));
            assert_eq!(path        , *text_ctrl   .file_path().deref()  );
            assert_eq!(another_path, *another_ctrl.file_path().deref()  );
        });
    }

    #[wasm_bindgen_test]
    fn obtain_text_controller_for_module() {
        let transport       = MockTransport::new();
        let mut test        = TestWithMockedTransport::set_up(&transport);
        test.run_test(async move {
            let project_ctrl = controller::Project::new_running(transport);
            let path         = ModuleLocation::new("test").to_path();
            let text_ctrl    = project_ctrl.text_controller(path.clone()).await.unwrap();
            let content      = text_ctrl.read_content().await.unwrap();
            assert_eq!("2 + 2", content.as_str());
        });
        test.when_stalled_send_response("2 + 2");
    }
}
