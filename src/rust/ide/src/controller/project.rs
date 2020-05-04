//! Project controller.
//!
//! Responsible for owning any remote connection clients, and providing controllers for specific
//! files and modules. Expected to live as long as the project remains open in the IDE.

use crate::prelude::*;

use enso_protocol::file_manager as fmc;
use enso_protocol::project_manager as pmc;
use json_rpc::Transport;
use parser::Parser;
use crate::transport::web::WebSocket;
use enso_protocol::project_manager::IpWithSocket;


// ==========================
// === Project Controller ===
// ==========================



/// Project controller's state.
#[derive(Debug)]
pub struct Handle {
    /// File Manager Client.
    pub file_manager : Option<Rc<fmc::Client>>,
    /// Project Manager Client.
    pub project_manager : Rc<pmc::Client>,
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
    pub async fn new(project_manager_transport:impl Transport + 'static) -> Self {
        let project_manager = Rc::new(pmc::Client::new(project_manager_transport));
        let file_manager    = None;
        let module_registry = default();
        let parser          = Parser::new_or_panic();
        let root_id         = default();
        Handle {root_id,file_manager,project_manager,module_registry,parser}
    }

    /// Creates a new project controller. Schedules all necessary execution with
    /// the global executor.
    pub async fn new_running(project_manager_transport:impl Transport + 'static) -> Self {
        let mut ret = Self::new(project_manager_transport).await;
        crate::executor::global::spawn(ret.project_manager.runner());
        let address = ret.open_most_recent_project().await.expect("Couldn't open project.");
        let error   = "Couldn't connect to language server.";
        ret.connect_to_language_server(address).await.expect(error);
        ret
    }

    /// Open most recent project or create a new project if none exists.
    pub async fn open_most_recent_project(&mut self) -> FallibleResult<IpWithSocket> {
        use pmc::API;
        let mut response = self.project_manager.list_recent_projects(1).await?;
        let project_id = if let Some(project) = response.projects.pop() {
            project.id
        } else {
            self.project_manager.create_project("InitialProject".into()).await?.project_id
        };
        Ok(self.project_manager.open_project(project_id).await?.language_server_address)
    }

    /// Connect to language server.
    pub async fn connect_to_language_server(&mut self, address:IpWithSocket) -> FallibleResult<()> {
        use fmc::API;
        let endpoint               = format!("ws://{}:{}",address.host,address.port);
        let file_manager_transport = WebSocket::new_opened(endpoint).await?;
        let file_manager           = Rc::new(fmc::Client::new(file_manager_transport));
        crate::executor::global::spawn(file_manager.runner());

        //FIXME[dg]: We need to make use of a proper client ID. I am still not sure where it
        // should come from so clarification is needed.
        let client_id = default();
        let response  = file_manager.init_protocol_connection(client_id).await;
        let response  = response.expect("Couldn't get project content roots.");
        //FIXME[dg]: We will make use of the first available `root_id`s, but we should expand this
        // logic to make use of the all available `root_id`s.
        self.file_manager = Some(file_manager);
        self.root_id      = response.content_roots[0];
        Ok(())
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
        let fm     = self.file_manager.clone().expect("Couldn't get file manager.");
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

    use enso_protocol::file_manager as fmc;
    use json_rpc::test_util::transport::mock::MockTransport;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;


    wasm_bindgen_test_configure!(run_in_browser);



    #[wasm_bindgen_test]
    fn obtain_module_controller() {
        let transport = MockTransport::new();
        let mut test  = TestWithMockedTransport::set_up(&transport);
        test.run_test(async move {
            let project      = controller::Project::new_running(transport).await;
            let path         = fmc::Path{root_id:default(),segments:vec!["TestLocation".into()]};
            let another_path = fmc::Path{root_id:default(),segments:vec!["TestLocation2".into()]};

            let module         = project.module_controller(path.clone()).await.unwrap();
            let same_module    = project.module_controller(path.clone()).await.unwrap();
            let another_module = project.module_controller(another_path.clone()).await.unwrap();

            assert_eq!(path,    module.path);
            assert_eq!(another_path, another_module.path);
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
            let root_id             = default();
            let path                = fmc::Path{root_id,segments:vec!["TestPath".into()]};
            let another_path        = fmc::Path{root_id,segments:vec!["TestPath2".into()]};

            let text_ctrl    = project_ctrl.text_controller(path.clone()).await.unwrap();
            let another_ctrl = project_ctrl.text_controller(another_path.clone()).await.unwrap();

            let file_manager = project_ctrl.file_manager.expect("Couldn't get file manager.");

            assert!(Rc::ptr_eq(&file_manager,&text_ctrl.file_manager()));
            assert!(Rc::ptr_eq(&file_manager,&another_ctrl.file_manager()));
            assert_eq!(path        , *text_ctrl   .file_path().deref()  );
            assert_eq!(another_path, *another_ctrl.file_path().deref()  );
        });
    }

    #[wasm_bindgen_test]
    fn obtain_text_controller_for_module() {
        let transport       = MockTransport::new();
        let mut test        = TestWithMockedTransport::set_up(&transport);
        test.run_test(async move {
            let project_ctrl = controller::Project::new_running(transport).await;
            let path         = fmc::Path{root_id:default(),segments:vec!["test".into()]};
            let text_ctrl    = project_ctrl.text_controller(path.clone()).await.unwrap();
            let content      = text_ctrl.read_content().await.unwrap();
            assert_eq!("2 + 2", content.as_str());
        });
        test.when_stalled_send_response("2 + 2");
    }
}
