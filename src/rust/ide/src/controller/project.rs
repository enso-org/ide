//! Project controller.
//!
//! Responsible for owning any remote connection clients, and providing controllers for specific
//! files and modules. Expected to live as long as the project remains open in the IDE.

use crate::prelude::*;

use crate::controller::FilePath;
use crate::controller::Visualization;

use enso_protocol::language_server;
use enso_protocol::binary;
use parser::Parser;
use crate::model::synchronized::ExecutionContext;
use crate::model::execution_context::VisualizationUpdateData;


// ==========================
// === Project Controller ===
// ==========================

type ModulePath = controller::module::Path;
type ExecutionCtxId = model::execution_context::Id;

/// Project controller's state.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct Handle {
    pub project_name        : Rc<String>,
    pub language_server_rpc : Rc<language_server::Connection>,
    pub visualization       : Visualization,
    pub language_server_bin : Rc<binary::Connection>,
    pub module_registry     : Rc<model::registry::Registry<ModulePath,model::synchronized::Module>>,
    pub execution_contexts  : Rc<RefCell<HashMap<ExecutionCtxId, Weak<model::synchronized::ExecutionContext>>>>,
    pub parser              : Parser,
    pub logger              : Logger,
}

impl Handle {
    /// Create a new project controller.
    pub fn new
    ( parent                 : &Logger
    , language_server_client : language_server::Connection
    , mut language_server_binary : binary::Connection
    , project_name           : impl Str
    ) -> Self {
        let logger = parent.sub("Project Controller");
        info!(logger,"Creating a project controller for project {project_name.as_ref()}");
        let embedded_visualizations = default();
        let language_server_rpc     = Rc::new(language_server_client);
        let language_server         = language_server_rpc.clone();
        let visualization           = Visualization::new(language_server,embedded_visualizations);
        let binary_protocol_events = language_server_binary.event_stream();
        let ret = Handle {
            project_name        : Rc::new(project_name.into()),
            module_registry     : default(),
            execution_contexts  : default(),
            parser              : Parser::new_or_panic(),
            language_server_rpc,
            language_server_bin : Rc::new(language_server_binary),
            logger,
            visualization,
        };

        let logger     = ret.logger.clone_ref();
        let weak_execution_contexts = Rc::downgrade(&ret.execution_contexts);
        crate::executor::global::spawn(binary_protocol_events.for_each(move |event| {
            debug!(logger, "Received an event from the binary protocol: {event:?}");
            use enso_protocol::binary::client::Event;
            use enso_protocol::binary::Notification;
            match event {
                Event::Notification(Notification::VisualizationUpdate {context,data}) => {
                    let execution_contexts = weak_execution_contexts.upgrade();

                    if let Some(execution_context) = execution_contexts.and_then(|contexts| {
                        let contexts = contexts.borrow();
                        contexts.get(&context.context_id)?.upgrade()
                    }) {
                        debug!(logger, "Dispatching visualization update through context {context.context_id}");
                        let update_data = VisualizationUpdateData::new(data);
                        execution_context.dispatch_update(context.visualization_id,update_data);
                    } else {
                        warning!(logger, "Unexpected visualization update for execution context \
                    {context.context_id}. No information about such context, cannot route the \
                    update to any visualization.");
                    }
                }
                _ => {}
            }
            futures::future::ready(())
        }));

        ret
    }

    /// Returns a text controller for a given file path.
    ///
    /// It supports both modules and plain text files.
    pub async fn text_controller(&self, path:FilePath) -> FallibleResult<controller::Text> {
        if let Some(path) = controller::module::Path::from_file_path(path.clone()) {
            info!(self.logger,"Obtaining controller for module {path}");
            let module = self.module_controller(path).await?;
            Ok(controller::Text::new_for_module(module))
        } else {
            let ls = self.language_server_rpc.clone_ref();
            info!(self.logger,"Obtaining controller for plain text {path}");
            Ok(controller::Text::new_for_plain_text(path,ls))
        }
    }

    /// Returns a module controller which have module opened from file.
    pub async fn module_controller
    (&self, path:ModulePath) -> FallibleResult<controller::Module> {
        info!(self.logger,"Obtaining module controller for {path}");
        let model_loader = self.load_module(path.clone());
        let model        = self.module_registry.get_or_load(path.clone(),model_loader).await?;
        Ok(self.module_controller_with_model(path,model))
    }

    /// Creates a path describing a module in this project.
    pub fn module_path
    (&self, segments:impl IntoIterator<Item:AsRef<str>>) -> FallibleResult<ModulePath> {
        let root_id = self.language_server_rpc.content_root();
        let path    = language_server::Path::new(root_id,segments);
        Ok(path.try_into()?)
    }

    pub fn qualified_module_name(&self, path:&controller::module::Path) -> String {
        path.qualified_name(self.project_name.deref())
    }

    fn module_controller_with_model
    (&self, path:ModulePath, model:Rc<model::synchronized::Module>)
    -> controller::Module {
        let ls     = self.language_server_rpc.clone_ref();
        let parser = self.parser.clone_ref();
        controller::Module::new(&self.logger,path,model,ls,parser)
    }

    fn load_module(&self, path:ModulePath)
    -> impl Future<Output=FallibleResult<Rc<model::synchronized::Module>>> {
        let language_server = self.language_server_rpc.clone_ref();
        let parser          = self.parser.clone_ref();
        model::synchronized::Module::open(path,language_server,parser)
    }

    pub async fn create_execution_context
    (&self
    , module_path:Rc<controller::module::Path>
    , root_definition:double_representation::definition::DefinitionName
    ) -> FallibleResult<Rc<model::synchronized::ExecutionContext>> {
        use model::synchronized::ExecutionContext;
        let ls_rpc  = self.language_server_rpc.clone_ref();
        let context = ExecutionContext::create(&self.logger,ls_rpc,module_path,root_definition);
        let context = context.await?;
        let id      = context.id();
        let context = Rc::new(context);
        self.register_execution_context(&context);
        Ok(context)
    }

    pub fn register_execution_context(&self, execution_context:&Rc<ExecutionContext>) {
        let id   = execution_context.id();
        let weak = Rc::downgrade(&execution_context);
        self.execution_contexts.borrow_mut().insert(id,weak);
    }

    fn process_binary_notification(&self, notification:enso_protocol::binary::Notification) {
        use enso_protocol::binary::Notification;
        match notification {
            Notification::VisualizationUpdate {context,data} => {
                let execution_contexts = with(self.execution_contexts.borrow(), |map| {
                    map.get(&context.context_id)?.upgrade()
                });
                if let Some(execution_context) = execution_contexts {
                    let update_data = VisualizationUpdateData::new(data);
                    execution_context.dispatch_update(context.visualization_id,update_data);
                } else {
                    warning!(self.logger, "Unexpected visualization update for execution context \
                    {context.context_id}. No information about such context, cannot route the \
                    update to any visualization.");
                }
            }
        }
    }
}



// ============
// === Test ===
// ============

#[cfg(test)]
mod test {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;

    use json_rpc::expect_call;
    use language_server::response;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use enso_protocol::language_server::CapabilityRegistration;
    use enso_protocol::types::Sha3_224;
    use crate::DEFAULT_PROJECT_NAME;


    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn obtain_module_controller() {
        let mut test  = TestWithLocalPoolExecutor::set_up();
        test.run_task(async move {
            let path         = ModulePath::from_mock_module_name("TestModule");
            let another_path = ModulePath::from_mock_module_name("TestModule2");

            let json_client = language_server::MockClient::default();
            mock_calls_for_opening_text_file(&json_client,path.file_path().clone(),"2+2");
            mock_calls_for_opening_text_file(&json_client,another_path.file_path().clone(),"22+2");
            let json_connection   = language_server::Connection::new_mock(json_client);
            let binary_connection = binary::Connection::new_mock(default());
            let project           = controller::Project::new(&default(),json_connection,
                binary_connection,DEFAULT_PROJECT_NAME);
            let module            = project.module_controller(path.clone()).await.unwrap();
            let same_module       = project.module_controller(path.clone()).await.unwrap();
            let another_module    = project.module_controller(another_path.clone()).await.unwrap();

            assert_eq!(path,         *module.path);
            assert_eq!(another_path, *another_module.path);
            assert!(Rc::ptr_eq(&module.model, &same_module.model));
        });
    }

    #[wasm_bindgen_test]
    fn obtain_plain_text_controller() {
        TestWithLocalPoolExecutor::set_up().run_task(async move {
            let json_connection   = language_server::Connection::new_mock(default());
            let binary_connection = binary::Connection::new_mock(default());
            let project_ctrl      = controller::Project::new(&default(),json_connection,
                binary_connection,DEFAULT_PROJECT_NAME);
            let root_id           = default();
            let path              = FilePath::new(root_id,&["TestPath"]);
            let another_path      = FilePath::new(root_id,&["TestPath2"]);

            let text_ctrl       = project_ctrl.text_controller(path.clone()).await.unwrap();
            let another_ctrl    = project_ctrl.text_controller(another_path.clone()).await.unwrap();
            let language_server = project_ctrl.language_server_rpc;

            assert!(Rc::ptr_eq(&language_server,&text_ctrl.language_server()));
            assert!(Rc::ptr_eq(&language_server,&another_ctrl.language_server()));
            assert_eq!(path        , *text_ctrl   .file_path().deref()  );
            assert_eq!(another_path, *another_ctrl.file_path().deref()  );
        });
    }

    #[wasm_bindgen_test]
    fn obtain_text_controller_for_module() {
        let mut test = TestWithLocalPoolExecutor::set_up();
        test.run_task(async move {
            let file_name    = format!("Test.{}",constants::LANGUAGE_FILE_EXTENSION);
            let path         = FilePath::new(default(),&[file_name]);

            let json_client = language_server::MockClient::default();
            mock_calls_for_opening_text_file(&json_client,path.clone(),"2 + 2");
            let json_connection   = language_server::Connection::new_mock(json_client);
            let binary_connection = binary::Connection::new_mock(default());
            let project_ctrl      = controller::Project::new(&default(),json_connection,
                binary_connection,DEFAULT_PROJECT_NAME);
            let text_ctrl         = project_ctrl.text_controller(path.clone()).await.unwrap();
            let content           = text_ctrl.read_content().await.unwrap();
            assert_eq!("2 + 2", content.as_str());
        });
    }

    fn mock_calls_for_opening_text_file
    (client:&language_server::MockClient, path:language_server::Path, content:&str) {
        let content          = content.to_string();
        let current_version  = Sha3_224::new(content.as_bytes());
        let write_capability = CapabilityRegistration::create_can_edit_text_file(path.clone());
        let write_capability = Some(write_capability);
        let open_response    = response::OpenTextFile {content,current_version,write_capability};
        expect_call!(client.open_text_file(path=path.clone()) => Ok(open_response));
        client.expect.apply_text_file_edit(|_| Ok(()));
        expect_call!(client.close_text_file(path) => Ok(()));
    }
}
