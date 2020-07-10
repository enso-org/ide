//! Project controller.
//!
//! Responsible for owning any remote connection clients, and providing controllers for specific
//! files and modules. Expected to live as long as the project remains open in the IDE.
pub mod synchronized;

use crate::prelude::*;

use crate::model::module;

use enso_protocol::binary;
use enso_protocol::language_server;
use mockall::automock;
use parser::Parser;
use uuid::Uuid;



// =============
// === Model ===
// =============

#[automock]
pub trait API:Debug {
    /// Project's name
    fn name(&self) -> ImString;

    fn json_rpc(&self) -> Rc<language_server::Connection>;

    fn binary_rpc(&self) -> Rc<binary::Connection>;

    fn parser(&self) -> &Parser;

    fn visualization(&self) -> &controller::Visualization;

    /// Returns a model of module opened from file.
    fn module<'a>
    (&'a self, path:crate::model::module::Path) -> BoxFuture<'a,FallibleResult<model::Module>>;

    /// Creates a new execution context with given definition as a root; and registers the context
    /// for receiving update.
    fn create_execution_context<'a>
    (&'a self, root_definition:language_server::MethodPointer)
    -> BoxFuture<'a,FallibleResult<model::ExecutionContext>>;

    /// Returns the primary content root id for this project.
    fn content_root_id(&self) -> Uuid {
        self.json_rpc().content_root()
    }

    /// Generates full module's qualified name that includes the leading project name segment.
    fn qualified_module_name(&self, path:&model::module::Path) -> crate::model::module::QualifiedName {
        module::QualifiedName::from_path(path,self.name().deref())
    }
}

impl Debug for MockAPI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Mocked Project Model")
    }
}

pub type Synchronized = synchronized::Project;

#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Project {
    rc:Rc<dyn API>
}

impl<P:API+'static> From<P> for Project {
    fn from(project:P) -> Self {
        Project {
            rc : Rc::new(project),
        }
    }
}



// ============
// === Test ===
// ============

// #[cfg(test)]
// pub mod test {
//     use super::*;
//
//     use crate::controller::FilePath;
//     use crate::constants::DEFAULT_PROJECT_NAME;
//     use crate::executor::test_utils::TestWithLocalPoolExecutor;
//
//     use json_rpc::expect_call;
//     use language_server::response;
//     use wasm_bindgen_test::wasm_bindgen_test;
//     use wasm_bindgen_test::wasm_bindgen_test_configure;
//     use enso_protocol::language_server::CapabilityRegistration;
//     use enso_protocol::language_server::Event;
//     use enso_protocol::language_server::Notification;
//     use enso_protocol::types::Sha3_224;
//     use utils::test::future::FutureTestExt;
//
//
//     wasm_bindgen_test_configure!(run_in_browser);
//
//     /// Sets up project controller using mock Language Server clients.
//     /// Passed functions should be used to setup expectations upon the mock clients.
//     /// Additionally, an `event_stream` expectation will be setup for a binary protocol, and
//     /// `get_suggestion_database` for json protocol, as
//     /// project controller always calls them.
//     pub fn setup_mock_project
//     ( setup_mock_json   : impl FnOnce(&mut language_server::MockClient)
//     , setup_mock_binary : impl FnOnce(&mut enso_protocol::binary::MockClient)
//     ) -> Project {
//         let mut json_client   = language_server::MockClient::default();
//         let mut binary_client = enso_protocol::binary::MockClient::default();
//         binary_client.expect_event_stream().return_once(|| {
//             futures::stream::empty().boxed_local()
//         });
//         let initial_suggestions_db = language_server::response::GetSuggestionDatabase {
//             entries: vec![],
//             current_version: 0
//         };
//         expect_call!(json_client.get_suggestions_database() => Ok(initial_suggestions_db));
//         let capability_reg = CapabilityRegistration::create_receives_suggestions_database_updates();
//         let method         = capability_reg.method;
//         let options        = capability_reg.register_options;
//         expect_call!(json_client.acquire_capability(method,options) => Ok(()));
//
//         setup_mock_json(&mut json_client);
//         setup_mock_binary(&mut binary_client);
//         let json_connection   = language_server::Connection::new_mock(json_client);
//         let binary_connection = binary::Connection::new_mock(binary_client);
//         let logger            = Logger::default();
//         let mut project_fut   = model::Project::from_connections(logger,json_connection,
//             binary_connection,DEFAULT_PROJECT_NAME).boxed_local();
//         project_fut.expect_ready().unwrap()
//     }
//
//     #[wasm_bindgen_test]
//     fn obtain_module_controller() {
//         let mut test  = TestWithLocalPoolExecutor::set_up();
//         test.run_task(async move {
//             use controller::Module;
//
//             let path         = module::Path::from_mock_module_name("TestModule");
//             let another_path = module::Path::from_mock_module_name("TestModule2");
//
//             let project = setup_mock_project(|ls_json| {
//                 mock_calls_for_opening_text_file(ls_json,path.file_path().clone(),"2+2");
//                 mock_calls_for_opening_text_file(ls_json,another_path.file_path().clone(),"22+2");
//             }, |_| {});
//             let log               = Logger::new("Test");
//             let module            = Module::new(&log,path.clone(),&project).await.unwrap();
//             let same_module       = Module::new(&log,path.clone(),&project).await.unwrap();
//             let another_module    = Module::new(&log,another_path.clone(),&project).await.unwrap();
//
//             assert_eq!(path,         module.model.path);
//             assert_eq!(another_path, another_module.model.path);
//             assert!(Rc::ptr_eq(&module.model, &same_module.model));
//         });
//     }
//
//     #[wasm_bindgen_test]
//     fn obtain_plain_text_controller() {
//         TestWithLocalPoolExecutor::set_up().run_task(async move {
//
//             let project      = setup_mock_project(|_|{}, |_|{});
//             let root_id      = default();
//             let path         = FilePath::new(root_id,&["TestPath"]);
//             let another_path = FilePath::new(root_id,&["TestPath2"]);
//
//             let log             = Logger::new("Test");
//             let text_ctrl       = controller::Text::new(&log,&project,path.clone());
//             let text_ctrl       = text_ctrl.await.unwrap();
//             let another_ctrl    = controller::Text::new(&log,&project,another_path.clone());
//             let another_ctrl    = another_ctrl.await.unwrap();
//             let language_server = project.language_server_rpc;
//
//             assert!(Rc::ptr_eq(&language_server,&text_ctrl.language_server()));
//             assert!(Rc::ptr_eq(&language_server,&another_ctrl.language_server()));
//             assert_eq!(path        , *text_ctrl   .file_path().deref()  );
//             assert_eq!(another_path, *another_ctrl.file_path().deref()  );
//         });
//     }
//
//     #[wasm_bindgen_test]
//     fn obtain_text_controller_for_module() {
//         let mut test = TestWithLocalPoolExecutor::set_up();
//         test.run_task(async move {
//             let module_path  = module::Path::from_mock_module_name("Test");
//             let file_path    = module_path.file_path();
//             let project      = setup_mock_project(|mock_json_client| {
//                 mock_calls_for_opening_text_file(mock_json_client,file_path.clone(),"2 + 2");
//             }, |_| {});
//             let log       = Logger::new("Test");
//             let text_ctrl = controller::Text::new(&log,&project,file_path.clone()).await.unwrap();
//             let content   = text_ctrl.read_content().await.unwrap();
//             assert_eq!("2 + 2", content.as_str());
//         });
//     }
//
//     /// This tests checks mainly if:
//     /// * project controller correctly creates execution context
//     /// * created execution context appears in the registry
//     /// * project controller correctly dispatches the LS notification with type information
//     /// * the type information is correctly recorded and available in the execution context
//     #[wasm_bindgen_test]
//     fn execution_context_management() {
//         // Setup project controller and mock LS client expectations.
//         let mut test   = TestWithLocalPoolExecutor::set_up();
//         let data       = model::synchronized::execution_context::tests::MockData::new();
//         let mut sender = futures::channel::mpsc::unbounded().0;
//         let project    = setup_mock_project(|mock_json_client| {
//             data.mock_create_push_destroy_calls(mock_json_client);
//             sender = mock_json_client.setup_events();
//             mock_json_client.require_all_calls();
//         }, |_| {});
//
//         // No context present yet.
//         let no_op = |_| Ok(());
//         let result1 = project.execution_contexts.with_context(data.context_id,no_op);
//         assert!(result1.is_err());
//
//         // Create execution context.
//         let execution   = project.create_execution_context(data.main_method_pointer());
//         let execution   = test.expect_completion(execution).unwrap();
//
//         // Now context is in registry.
//         let result1 = project.execution_contexts.with_context(data.context_id,no_op);
//         assert!(result1.is_ok());
//
//         // Context has no information about type.
//         let notification   = data.mock_values_computed_update();
//         let value_update   = &notification.updates[0];
//         let expression_id  = value_update.id;
//         let value_registry = execution.computed_value_info_registry();
//         assert!(value_registry.get(&expression_id).is_none());
//
//         // Send notification with type information.
//         let event = Event::Notification(Notification::ExpressionValuesComputed(notification.clone()));
//         sender.unbounded_send(event).unwrap();
//         test.run_until_stalled();
//
//         // Context now has the information about type.
//         let value_info = value_registry.get(&expression_id).unwrap();
//         assert_eq!(value_info.typename, value_update.typename.clone().map(ImString::new));
//         assert_eq!(value_info.method_pointer, value_update.method_call.clone().map(Rc::new));
//     }
//
//     fn mock_calls_for_opening_text_file
//     (client:&language_server::MockClient, path:language_server::Path, content:&str) {
//         let content          = content.to_string();
//         let current_version  = Sha3_224::new(content.as_bytes());
//         let write_capability = CapabilityRegistration::create_can_edit_text_file(path.clone());
//         let write_capability = Some(write_capability);
//         let open_response    = response::OpenTextFile {content,current_version,write_capability};
//         expect_call!(client.open_text_file(path=path.clone()) => Ok(open_response));
//         client.expect.apply_text_file_edit(|_| Ok(()));
//         expect_call!(client.close_text_file(path) => Ok(()));
//     }
// }
