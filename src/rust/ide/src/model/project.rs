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
