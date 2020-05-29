//! Module Controller.
//!
//! The module controller keeps cached module state (module state is AST+Metadata or equivalent),
//! and uses it for synchronizing state for text and graph representations. It provides method
//! for registering text and graph changes. If for example text represntation will be changed, there
//! will be notifications for both text change and graph change.

use crate::prelude::*;

use crate::controller::FilePath;
use crate::constants::LANGUAGE_FILE_DOT_EXTENSION;
use crate::constants::SOURCE_DIRECTORY;
use crate::double_representation::text::apply_code_change_to_id_map;

use ast;
use ast::HasIdMap;
use data::text::*;
use double_representation as dr;
use enso_protocol::language_server;
use enso_protocol::types::Sha3_224;
use failure::_core::fmt::Formatter;
use parser::Parser;




// ==============
// === Errors ===
// ==============

/// Happens if an empty segments list is provided as qualified module name.
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="No qualified name segments were provided.")]
pub struct EmptyQualifiedName;

/// Error returned when graph id invalid.
#[derive(Clone,Debug,Fail)]
#[fail(display="Invalid graph id: {:?}.",_0)]
pub struct InvalidGraphId(controller::graph::Id);

/// Failed attempt to tread a file path as a module path.
#[derive(Clone,Debug,Fail)]
#[fail(display = "The path `{}` is not a valid module path. {}",path,issue)]
pub struct InvalidModulePath {
    /// The path that is not a valid module path.
    path  : FilePath,
    /// The reason why the path is not a valid modile path.
    issue : ModulePathViolation
}

/// Describes possible reasons why a `FilePath` cannot be recognized as a `ModulePath`.
#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,Fail)]
pub enum ModulePathViolation {
    #[fail(display="The module filename should be capitalized.")]
    NonCapitalizedFileName,
    #[fail(display="The path contains an empty segment which is not allowed.")]
    ContainsEmptySegment,
    #[fail(display="The file path does not contain any segments, while it should be non-empty.")]
    ContainsNoSegments,
    #[fail(display="The module file path should start with the sources directory.")]
    NotInSourceDirectory,
    #[fail(display="The module file must have a proper language extension.")]
    WrongFileExtension,
}



// ============
// === Path ===
// ============

/// Path identifying module's file in the Language Server.
///
/// The `file_path` contains at least two segments:
/// * the first one is a source directory in the project (see `SOURCE_DIRECTORY`);
/// * the last one is a source file with the module's contents.
#[derive(Clone,Debug,Eq,Hash,PartialEq,Shrinkwrap)]
pub struct Path {
    file_path : FilePath,
}

impl Path {
    /// Create a path from the file path. Returns Err if given path is not a valid module file.
    pub fn from_file_path(file_path:FilePath) -> Result<Self,InvalidModulePath> {
        use ModulePathViolation::*;
        let error             = |issue| InvalidModulePath {path:file_path.clone(),issue};
        let correct_extension = file_path.extension() == Some(constants::LANGUAGE_FILE_EXTENSION);
        correct_extension.ok_or_else(|| error(WrongFileExtension))?;
        let file_name       = file_path.file_name().ok_or(error(ContainsNoSegments))?;
        let name_first_char = file_name.chars().next().ok_or(error(ContainsEmptySegment))?;
        name_first_char.is_uppercase().ok_or_else(|| error(NonCapitalizedFileName))?;
        let is_in_src = file_path.segments.first().contains_if(|name| *name == SOURCE_DIRECTORY);
        is_in_src.ok_or_else(|| error(NotInSourceDirectory))?;
        Ok(Path {file_path})
    }

    /// Creates a module path from the module's qualified name segments.
    /// Name segments should only cover the module names, excluding the project name.
    ///
    /// E.g. `["Main"]` -> `//root_id/src/Main.enso`
    pub fn from_name_segments
    (root_id:Uuid, name_segments:impl IntoIterator<Item:AsRef<str>>) -> FallibleResult<Path> {
        let mut segments : Vec<String> = vec![SOURCE_DIRECTORY.into()];
        segments.extend(name_segments.into_iter().map(|segment| segment.as_ref().to_string()));
        let module_file = segments.last_mut().ok_or(EmptyQualifiedName)?;
        module_file.push_str(LANGUAGE_FILE_DOT_EXTENSION);
        let file_path = FilePath {root_id,segments} ;
        Ok(Path {file_path})
    }

    /// Get the file path.
    pub fn file_path(&self) -> &FilePath {
        &self.file_path
    }

    /// Gives the file name for the given module name.
    ///
    /// E.g. "Main" -> "Main.enso"
    pub fn name_to_file_name(name:impl Str) -> String {
        format!("{}.{}",name.as_ref(),constants::LANGUAGE_FILE_EXTENSION)
    }

    /// Get the module name from path.
    ///
    /// The module name is a filename without extension.
    pub fn module_name(&self) -> &str {
        // The file stem existence should be checked during construction.
        self.file_path.file_stem().unwrap()
    }

    /// Create a module path consisting of a single segment, based on a given module name.
    /// The `default` is used for a root id.
    pub fn from_mock_module_name(name:impl Str) -> Self {
        let file_name   = Self::name_to_file_name(name);
        let src_dir     = SOURCE_DIRECTORY.to_string();
        let file_path   = FilePath::new(default(),&[src_dir,file_name]);
        Self::from_file_path(file_path).unwrap()
    }
}

impl TryFrom<FilePath> for Path {
    type Error = InvalidModulePath;

    fn try_from(value:FilePath) -> Result<Self, Self::Error> {
        Path::from_file_path(value)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.file_path, f)
    }
}



// ===========================
// === ModuleQualifiedName ===
// ===========================

/// Module's qualified name is used in some of the Language Server's APIs, like
/// `VisualisationConfiguration`.
///
/// Qualified name is constructed as follows:
/// `ProjectName.<directories_between_src_and_enso_file>.<file_without_ext>`
///
/// See https://dev.enso.org/docs/distribution/packaging.html for more information about the
/// package structure.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct QualifiedName(String);

impl QualifiedName {
    /// Obtains a module's full qualified name from its path and the project name.
    pub fn from_path(path:&Path, project_name:impl Str) -> QualifiedName {
        let project_name        = std::iter::once(project_name.as_ref());
        let non_src_directories = &path.file_path.segments[1..path.file_path.segments.len()-1];
        let directories_strs    = non_src_directories.iter().map(|string| string.as_str());
        let module_name         = std::iter::once(path.module_name());
        let name                = project_name.chain(directories_strs.chain(module_name)).join(".");
        QualifiedName(name)
    }
}



// =========================
// === Module Controller ===
// =========================

/// A Handle for Module Controller.
///
/// This struct contains all information and handles to do all module controller operations.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct Handle {
    pub path            : Rc<Path>,
    pub model           : Rc<model::synchronized::Module>,
    pub language_server : Rc<language_server::Connection>,
    pub parser          : Parser,
    pub logger          : Logger,
}

impl Handle {
    /// Create a module controller for given path.
    ///
    /// This function won't load module from file - it just get the state in `model` argument.
    pub fn new
    ( parent          : &Logger
    , path            : Path
    , model           : Rc<model::synchronized::Module>
    , language_server : Rc<language_server::Connection>
    , parser          : Parser
    ) -> Self {
        let logger = parent.sub(format!("Module Controller {}", path));
        let path   = Rc::new(path);
        Handle {path,model,language_server,parser,logger}
    }

    /// Save the module to file.
    pub fn save_file(&self) -> impl Future<Output=FallibleResult<()>> {
        let content = self.model.serialized_content();
        let path    = self.path.clone_ref();
        let ls      = self.language_server.clone();
        async move {
            let version = Sha3_224::new(content?.content.as_bytes());
            Ok(ls.client.save_text_file(path.file_path(),&version).await?)
        }
    }

    /// Updates AST after code change.
    ///
    /// May return Error when new code causes parsing errors, or when parsed code does not produce
    /// Module ast.
    pub fn apply_code_change(&self,change:TextChange) -> FallibleResult<()> {
        let mut id_map       = self.model.ast().id_map();
        let replaced_size    = change.replaced.end - change.replaced.start;
        let replaced_span    = Span::new(change.replaced.start,replaced_size);

        apply_code_change_to_id_map(&mut id_map,&replaced_span,&change.inserted);
        self.model.apply_code_change(change,&self.parser,id_map)
    }

    /// Read module code.
    pub fn code(&self) -> String {
        self.model.ast().repr()
    }

    /// Check if current module state is synchronized with given code. If it's not, log error,
    /// and update module state to match the `code` passed as argument.
    pub fn check_code_sync(&self, code:String) -> FallibleResult<()> {
        let my_code = self.code();
        if code != my_code {
            error!(self.logger,"The module controller ast was not synchronized with text editor \
                content!\n >>> Module: {my_code}\n >>> Editor: {code}");
            let actual_ast = self.parser.parse(code,default())?.try_into()?;
            self.model.update_ast(actual_ast);
        }
        Ok(())
    }

    /// Returns a graph controller for graph in this module's subtree identified by `id`.
    pub fn graph_controller(&self, id:dr::graph::Id) -> FallibleResult<controller::Graph> {
        controller::Graph::new(&self.logger, self.model.clone_ref(), self.parser.clone_ref(), id)
    }

    /// Returns a executed graph controller for graph in this module's subtree identified by id.
    /// The execution context will be rooted at definition of this graph.
    ///
    /// This function wont check if the definition under id exists.
    pub async fn executed_graph_controller_unchecked
    (&self, id:dr::graph::Id, project:&controller::Project)
    -> FallibleResult<controller::ExecutedGraph> {
        let definition_name = id.crumbs.last().cloned().ok_or_else(|| InvalidGraphId(id.clone()))?;
        let graph           = self.graph_controller_unchecked(id);
        let path            = self.path.clone_ref();
        let execution_ctx   = project.create_execution_context(path,definition_name).await?;
        Ok(controller::ExecutedGraph::new(graph,execution_ctx))
    }

    /// Returns a graph controller for graph in this module's subtree identified by `id` without
    /// checking if the graph exists.
    pub fn graph_controller_unchecked(&self, id:dr::graph::Id) -> controller::Graph {
        controller::Graph::new_unchecked(&self.logger, self.model.clone_ref(),
                                         self.parser.clone_ref(), id)
    }

    #[cfg(test)]
    pub fn new_mock
    ( path            : Path
    , code            : &str
    , id_map          : ast::IdMap
    , language_server : Rc<language_server::Connection>
    , parser          : Parser
    ) -> FallibleResult<Self> {
        let logger = Logger::new("Mocked Module Controller");
        let ast    = parser.parse(code.to_string(),id_map.clone())?.try_into()?;
        let model  = model::Module::new(ast, default());
        let model  = model::synchronized::Module::mock(path.clone(),model);
        let path   = Rc::new(path);
        Ok(Handle {path,model,language_server,parser,logger})
    }

    #[cfg(test)]
    pub fn expect_code(&self, expected_code:impl Str) {
        let code = self.code();
        assert_eq!(code,expected_code.as_ref());
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    use crate::controller::module::QualifiedName as ModuleQualifiedName;
    use crate::executor::test_utils::TestWithLocalPoolExecutor;

    use ast;
    use ast::BlockLine;
    use ast::Ast;
    use data::text::Span;
    use parser::Parser;
    use uuid::Uuid;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    fn module_path_conversion() {
        let path = FilePath::new(default(), &["src","Main.enso"]);
        assert!(Path::from_file_path(path).is_ok());

        let path = FilePath::new(default(), &["src","Main.txt"]);
        assert!(Path::from_file_path(path).is_err());

        let path = FilePath::new(default(), &["src","main.txt"]);
        assert!(Path::from_file_path(path).is_err());
    }

    #[test]
    fn module_path_validation() {
        assert!(Path::from_file_path(FilePath::new(default(), &["src", "Main.enso"])).is_ok());

        assert!(Path::from_file_path(FilePath::new(default(), &["surce", "Main.enso"])).is_err());
        assert!(Path::from_file_path(FilePath::new(default(), &["src", "Main"])).is_err());
        assert!(Path::from_file_path(FilePath::new(default(), &["src", ""])).is_err());
        assert!(Path::from_file_path(FilePath::new(default(), &["src", "main.enso"])).is_err());
    }

    #[test]
    fn module_qualified_name() {
        let project_name = "P";
        let root_id      = default();
        let file_path    = FilePath::new(root_id, &["src", "Foo", "Bar.enso"]);
        let module_path  = Path::from_file_path(file_path).unwrap();
        let qualified    = ModuleQualifiedName::from_path(&module_path,project_name);
        assert_eq!(*qualified, "P.Foo.Bar");
    }

    #[wasm_bindgen_test]
    fn update_ast_after_text_change() {
        TestWithLocalPoolExecutor::set_up().run_task(async {
            let ls       = language_server::Connection::new_mock_rc(default());
            let parser   = Parser::new().unwrap();
            let location = Path::from_mock_module_name("Test");

            let uuid1    = Uuid::new_v4();
            let uuid2    = Uuid::new_v4();
            let uuid3    = Uuid::new_v4();
            let uuid4    = Uuid::new_v4();
            let module   = "2+2";
            let id_map   = ast::IdMap::new(vec!
                [ (Span::new(Index::new(0),Size::new(1)),uuid1)
                , (Span::new(Index::new(1),Size::new(1)),uuid2)
                , (Span::new(Index::new(2),Size::new(1)),uuid3)
                , (Span::new(Index::new(0),Size::new(3)),uuid4)
                ]);

            let controller = Handle::new_mock(location,module,id_map,ls,parser).unwrap();

            // Change code from "2+2" to "22+2"
            let change = TextChange::insert(Index::new(1),"2".to_string());
            controller.apply_code_change(change).unwrap();
            let expected_ast = Ast::new_no_id(ast::Module {
                lines: vec![BlockLine {
                    elem: Some(Ast::new(ast::Infix {
                        larg : Ast::new(ast::Number{base:None, int:"22".to_string()}, Some(uuid1)),
                        loff : 0,
                        opr  : Ast::new(ast::Opr {name:"+".to_string()}, Some(uuid2)),
                        roff : 0,
                        rarg : Ast::new(ast::Number{base:None, int:"2".to_string()}, Some(uuid3)),
                    }, Some(uuid4))),
                    off: 0
                }]
            });
            assert_eq!(expected_ast, controller.model.ast().into());
        });
    }
}
