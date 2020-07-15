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

pub type Project      = Rc<dyn API>;
pub type Synchronized = synchronized::Project;



// ============
// === Test ===
// ============


//TODO[ao]: Those test should be put in Text Controller module.
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
// }
