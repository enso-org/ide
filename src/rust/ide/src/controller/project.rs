//! Project controller.
//!
//! Responsible for owning any remote connection clients, and providing controllers for specific
//! files and modules. Expected to live as long as the project remains open in the IDE.

use crate::prelude::*;

use enso_protocol::language_server;
use parser::Parser;



// ==========================
// === Project Controller ===
// ==========================

type ModulePath = controller::module::Path;

/// Project controller's state.
#[derive(Debug)]
pub struct Handle {
    /// Client of Language Server bound to this project.
    pub language_server_rpc:Rc<language_server::Connection>,
    /// Cache of module controllers.
    pub module_registry:Rc<model::module::registry::Registry>,
    /// Parser handle.
    pub parser:Parser,
}

impl Handle {

    /// Takes a LS client which has not initialized its connection so far.
    pub async fn from_unitialized_client(language_server:impl language_server::API + 'static) -> FallibleResult<Self> {
        let language_server_client = language_server::Connection::new(language_server).await;
        language_server_client.map(Self::new)
    }

    /// Create a new project controller.
    ///
    /// The remote connection should be already established.
    pub fn new(language_server_client:language_server::Connection) -> Self {
        let module_registry     = default();
        let parser              = Parser::new_or_panic();
        let language_server_rpc = Rc::new(language_server_client);
        Handle {language_server_rpc,module_registry,parser}
    }

    /// Returns a text controller for given file path.
    ///
    /// It may be a controller for both modules and plain text files.
    pub async fn text_controller
    (&self, path:language_server::Path) -> FallibleResult<controller::Text> {
        if is_path_to_module(&path) {
            println!("Obtaining controller for module {}", path);
            let module = self.module_controller(path).await?;
            Ok(controller::Text::new_for_module(module))
        } else {
            let ls = self.language_server_rpc.clone_ref();
            println!("Obtaining controller for plain text {}", path);
            Ok(controller::Text::new_for_plain_text(path,ls))
        }
    }

    /// Returns a module controller which have module opened from file.
    pub async fn module_controller
    (&self, path:ModulePath) -> FallibleResult<controller::Module> {
        println!("Obtaining module controller for {}", path);
        let model_loader = self.load_module(path.clone());
        let model        = self.module_registry.get_or_load(path.clone(),model_loader).await?;
        Ok(self.module_controller_with_model(path,model))
    }

    fn module_controller_with_model
    (&self, path:ModulePath, model:Rc<model::Module>)
    -> controller::Module {
        let ls     = self.language_server_rpc.clone_ref();
        let parser = self.parser.clone_ref();
        controller::Module::new(path,model,ls,parser)
    }

    async fn load_module(&self, path:ModulePath) -> FallibleResult<Rc<model::Module>> {
        let model  = Rc::<model::Module>::default();
        let module = self.module_controller_with_model(path,model.clone_ref());
        module.load_file().await.map(move |()| model)
    }
}

fn is_path_to_module(path:&language_server::Path) -> bool {
    let extension = path.extension();
    extension.contains(&constants::LANGUAGE_FILE_EXTENSION)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::controller::text::FilePath;
    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::transport::test_utils::TestWithMockedTransport;

    use json_rpc::test_util::transport::mock::MockTransport;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;


    wasm_bindgen_test_configure!(run_in_browser);


    #[test]
    fn is_path_to_module_test() {
        let path = language_server::Path::new(default(), &["src","Main.enso"]);
        assert!(is_path_to_module(&path));

        let path = language_server::Path::new(default(), &["src","Main.txt"]);
        assert_eq!(is_path_to_module(&path), false);
    }

    #[wasm_bindgen_test]
    fn obtain_module_controller() {
        let transport = MockTransport::new();
        let mut test  = TestWithMockedTransport::set_up(&transport);
        test.run_test(async move {
            let project      = controller::Project::from_unitialized_client(language_server::Client::new(transport)).await.unwrap();
            let path         = ModulePath{root_id:default(),segments:vec!["TestLocation".into()]};
            let another_path = ModulePath{root_id:default(),segments:vec!["TestLocation2".into()]};

            let module         = project.module_controller(path.clone()).await.unwrap();
            let same_module    = project.module_controller(path.clone()).await.unwrap();
            let another_module = project.module_controller(another_path.clone()).await.unwrap();

            assert_eq!(path,         *module.path);
            assert_eq!(another_path, *another_module.path);
            assert!(Rc::ptr_eq(&module.model, &same_module.model));
        });

        test.when_stalled_send_response("2 + 2");
        test.when_stalled_send_response("3+3");
    }

    #[wasm_bindgen_test]
    fn obtain_plain_text_controller() {
        let transport       = MockTransport::new();
        TestWithLocalPoolExecutor::set_up().run_task(async move {
            let project_ctrl        = controller::Project::from_unitialized_client(language_server::Client::new(transport)).await.unwrap();
            let root_id             = default();
            let path                = FilePath{root_id,segments:vec!["TestPath".into()]};
            let another_path        = FilePath{root_id,segments:vec!["TestPath2".into()]};

            let text_ctrl    = project_ctrl.text_controller(path.clone()).await.unwrap();
            let another_ctrl = project_ctrl.text_controller(another_path.clone()).await.unwrap();

            let language_server = project_ctrl.language_server_rpc;

            assert!(Rc::ptr_eq(&language_server,&text_ctrl.language_server()));
            assert!(Rc::ptr_eq(&language_server,&another_ctrl.language_server()));
            assert_eq!(path        , *text_ctrl   .file_path().deref()  );
            assert_eq!(another_path, *another_ctrl.file_path().deref()  );
        });
    }

    #[wasm_bindgen_test]
    fn obtain_text_controller_for_module() {
        ensogl::system::web::set_stdout();
        println!("Hello moje 1");
        let transport       = MockTransport::new();
        let mut test        = TestWithMockedTransport::set_up(&transport);
        test.run_test(async move {
            let project_ctrl = controller::Project::from_unitialized_client(language_server::Client::new(transport)).await.unwrap();
            let file_name    = format!("test.{}",constants::LANGUAGE_FILE_EXTENSION);
            let path         = ModulePath{root_id:default(),segments:vec![file_name]};
            let text_ctrl    = project_ctrl.text_controller(path.clone()).await.unwrap();
            let content      = text_ctrl.read_content().await.unwrap();
            assert_eq!("2 + 2", content.as_str());
            println!("Hello moje 2");
        });
        println!("Hello moje 3");
        test.when_stalled_send_response("2 + 2");
    }
}
