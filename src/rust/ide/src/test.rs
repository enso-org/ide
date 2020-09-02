//! Module for support code for writing tests.

/// Utilities for mocking IDE components.
#[cfg(test)]
pub mod mock {
    use crate::prelude::*;

    /// Data used to create mock IDE components.
    ///
    /// Contains a number of constants and functions building more complex structures from them.
    /// The purpose is to allow different parts of tests that mock different models using
    /// consistent data.
    #[allow(missing_docs)]
    pub mod data {
        use enso_protocol::language_server::Position;
        use uuid::Uuid;

        pub const ROOT_ID         : Uuid     = Uuid::from_u128(100);
        pub const PROJECT_NAME    : &str     = "MockProject";
        pub const MODULE_NAME     : &str     = "Mock_Module";
        pub const CODE            : &str     = "main = \n    2 + 2";
        pub const DEFINITION_NAME : &str     = "main";
        pub const TYPE_NAME       : &str     = "Mock_Type";
        pub const MAIN_FINISH     : Position = Position {line:1, character:9};
        pub const CONTEXT_ID      : Uuid     = Uuid::from_u128(0xFE);

        pub fn module_path() -> crate::model::module::Path {
            crate::model::module::Path::from_name_segments(ROOT_ID, &[MODULE_NAME]).unwrap()
        }

        pub fn module_qualified_name() -> crate::double_representation::module::QualifiedName {
            module_path().qualified_module_name(PROJECT_NAME)
        }

        pub fn definition_name() -> crate::double_representation::definition::DefinitionName {
            crate::double_representation::definition::DefinitionName::new_plain(DEFINITION_NAME)
        }

        pub fn graph_id() -> crate::double_representation::graph::Id {
            crate::double_representation::graph::Id::new_plain_name(DEFINITION_NAME)
        }

        pub fn suggestion_db() -> crate::model::SuggestionDatabase {
            crate::model::SuggestionDatabase::default()
        }
    }

    #[derive(Clone,Debug)]
    struct Unified {
        pub logger        : Logger,
        pub project_name  : String,
        pub module_path   : model::module::Path,
        pub graph_id      : double_representation::graph::Id,
        pub suggestions   : HashMap<model::suggestion_database::EntryId,model::suggestion_database::Entry>,
        pub context_id    : model::execution_context::Id,
        pub parser        : parser::Parser,
        code              : String,
        id_map            : ast::IdMap,
        metadata          : crate::model::module::Metadata,
        root_definition   : double_representation::definition::DefinitionName,
    }

    impl Unified {
        pub fn new() -> Self {
            use crate::test::mock::data::*;
            Unified {
                logger          : Logger::default(),
                project_name    : PROJECT_NAME.to_owned(),
                module_path     : module_path(),
                graph_id        : graph_id(),
                code            : CODE.to_owned(),
                suggestions     : default(),
                id_map          : default(),
                metadata        : default(),
                context_id      : CONTEXT_ID,
                root_definition : definition_name(),
                parser          : parser::Parser::new_or_panic(),
            }
        }

        pub fn module(&self) -> crate::model::Module {
            let ast    = self.parser.parse_module(self.code.clone(),self.id_map.clone()).unwrap();
            let module = crate::model::module::Plain::new(self.module_path.clone(),ast,self.metadata.clone());
            Rc::new(module)
        }

        pub fn module_qualified_name(&self) -> double_representation::module::QualifiedName {
            self.module_path.qualified_module_name(&self.project_name)
        }

        pub fn definition_id(&self) -> double_representation::definition::Id {
            double_representation::definition::Id::new_single_crumb(self.root_definition.clone())
        }

        pub fn method_pointer(&self) -> enso_protocol::language_server::MethodPointer {
            enso_protocol::language_server::MethodPointer {
                module          : self.module_qualified_name().to_string(),
                defined_on_type : self.module_path.module_name().to_string(),
                name            : self.root_definition.to_string(),
            }
        }

        /// Create a graph controller from the current mock data.
        pub fn graph(&self, module:model::Module, db:Rc<model::SuggestionDatabase>)
        -> crate::controller::Graph {
            let logger = Logger::new("Test");
            let id     = self.graph_id.clone();
            let parser = self.parser.clone_ref();
            crate::controller::Graph::new(logger,module,db,parser,id).unwrap()
        }

        pub fn execution_context(&self) -> model::ExecutionContext {
            let logger = Logger::sub(&self.logger,"Mocked Execution Context");
            Rc::new(model::execution_context::Plain::new(logger,self.method_pointer()))
        }

        pub fn project(&self, module:model::Module, execution_context:model::ExecutionContext)
        -> model::Project {
            let mut project = model::project::MockAPI::new();
            model::project::test::expect_parser(&mut project,&self.parser);
            model::project::test::expect_module(&mut project,module);
            model::project::test::expect_execution_ctx(&mut project,execution_context);
            // Root ID is needed to generate module path used to get the module.
            model::project::test::expect_root_id(&mut project,crate::test::mock::data::ROOT_ID);
            Rc::new(project)
        }

        pub fn bake(&self) -> Baked {
            let logger = Logger::default(); // TODO
            let module = self.module();
            let suggestion_db = Rc::new(model::SuggestionDatabase::new_from_entries(logger,
                &self.suggestions));
            let graph  = self.graph(module.clone_ref(), suggestion_db.clone_ref());
            let execution = self.execution_context();
            let method_ptr = self.method_pointer();
            let project = self.project(module.clone_ref(),execution.clone_ref());
            let executed_graph = controller::ExecutedGraph::new_internal(graph.clone_ref(),
                project.clone_ref(),execution.clone_ref());
            Baked {
                data : self.clone(),
                module,
                graph,
                executed_graph,
                execution,
                suggestion_db,
                project,
            }
        }
    }

    #[derive(Clone,Debug)]
    struct Baked {
        data           : Unified,
        module         : model::Module,
        graph          : controller::Graph,
        execution      : model::ExecutionContext,
        executed_graph : controller::ExecutedGraph,
        suggestion_db  : Rc<model::SuggestionDatabase>,
        project        : model::Project,
    }

    impl Baked {
        // pub fn module(&mut self) -> crate::model::Module {
        //     self.module.get_or_insert(self.data.module()).clone_ref()
        // }
        //
        // /// Create a graph controller from the current mock data.
        // pub fn graph(&mut self, module:model::Module) -> crate::controller::Graph {
        //     let module = self.module();
        //     self.data.graph(module)
        // }
    }

    pub fn indent(line:impl AsRef<str>) -> String {
        iformat!("    {line.as_ref()}")
    }

    pub fn main_from_lines(lines:impl IntoIterator<Item:AsRef<str>>) -> String {
        def_from_lines("main",lines)
    }

    pub fn def_from_lines
    (name:impl Display, lines:impl IntoIterator<Item:AsRef<str>>) -> String {
        let body = lines.into_iter().map(indent).join("\n");
        iformat!("{name} =\n{body}")
    }
}
