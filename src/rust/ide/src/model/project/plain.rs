use crate::prelude::*;
use crate::controller::Visualization;
use crate::model::*;
use parser::Parser;
use crate::model::module::Path;
use enso_protocol::language_server::{MethodPointer, Connection};
use enso_protocol::{binary, language_server};


// =============
// === Model ===
// =============

/// Project Model.
///
///
#[allow(missing_docs)]
#[derive(Debug)]
pub struct Project {
    pub name                : ImString,
    pub language_server_bin : Rc<binary::Connection>,
    pub language_server_rpc : Rc<language_server::Connection>,
    pub visualization       : Visualization,
    pub modules             : HashMap<module::Path,Module>,
    pub suggestion_db       : SuggestionDatabase,
    pub parser              : Parser,
    pub logger              : Logger,
}

impl<Module> model::project::API for Project
where Module : model::module::API {
    fn name(&self) -> ImString {
        self.name.clone_ref()
    }

    fn json_rpc(&self) -> Rc<language_server::Connection> {
        self.language_server_rpc.clone_ref()
    }

    fn binary_rpc(&self) -> Rc<binary::Connection> {
        self.language_server_bin.clone_ref()
    }

    fn parser(&self) -> &Parser {
        &self.parser
    }

    fn module(&self, path: Path) -> LocalBoxFuture<FallibleResult<Module>> {
        let module = self.modules.get(&path).expect(iformat!("Unexpected module loaded: {path}"));
        futures::future::ready(module).boxed_local()
    }

    fn create_execution_context(&self, root_definition: MethodPointer) -> LocalBoxFuture<FallibleResult<Rc<ExecutionContext>>> {

    }

    fn content_root_id(&self) -> Uuid {
        self.content_root_id
    }
}