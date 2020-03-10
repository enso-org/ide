//! Module Controller.
//!
//! The module controller keeps cached module state (module state is AST+Metadata or equivalent),
//! and uses it for synchronizing state for text and graph representations. It provides method
//! for registering text and graph changes. If for example text represntation will be changed, there
//! will be notifications for both text change and graph change.

use crate::prelude::*;

use crate::controller::FallibleResult;
use crate::double_representation::apply_code_change_to_id_map;

use ast::Ast;
use ast::HasIdMap;
use ast::HasRepr;
use ast::IdMap;
use data::text::Index;
use data::text::Size;
use data::text::Span;
use data::text::TextChangedNotification;
use file_manager_client as fmc;
use json_rpc::error::RpcError;
use parser::api::IsParser;
use parser::api::ModuleWithMetadata;
use parser::Parser;

use serde::Serialize;
use serde::Deserialize;
use shapely::shared;



// ============
// == Module ==
// ============

/// Parsed file / module with metadata
#[derive(Debug,Clone)]
pub struct Module {
    /// ast representation
    pub ast: Ast,
    /// strongly typed metadata
    pub metadata: Metadata
}

impl TryFrom<ModuleWithMetadata> for Module {
    type Error = serde_json::Error;

    fn try_from(value:ModuleWithMetadata) -> Result<Self,Self::Error> {
        let metadata = Metadata::deserialize(value.metadata)?;
        Ok(Module {ast:value.ast,metadata})
    }
}

impl From<Module> for ModuleWithMetadata {
    fn from(value:Module) -> Self {
        let metadata = serde_json::to_value(value.metadata).expect(
            "Should be possible to serialize metadata to json."
        );
        ModuleWithMetadata { ast:value.ast, metadata }
    }
}


/// Mapping between ID and metadata.
#[derive(Debug,Clone,Default,Deserialize,Serialize)]
pub struct Metadata {
    /// metadata used within ide
    pub ide  : IdeMetadata,
    #[serde(flatten)]
    /// metadata used by anyone else - i.e. language server
    rest : HashMap<String,serde_json::Value>,
}


/// Ide related metadata.
#[derive(Debug,Clone,Default,Deserialize,Serialize)]
pub struct IdeMetadata {}



// =======================
// === Module Location ===
// =======================

/// Structure uniquely identifying module location in the project.
/// Mappable to filesystem path.
#[derive(Clone,Debug,Display,Eq,Hash,PartialEq)]
pub struct Location(pub String);

impl Location {
    /// Get the module location from filesystem path. Returns None if path does not lead to
    /// module file.
    pub fn from_path(path:&fmc::Path) -> Option<Self> {
        // TODO [ao] See function `to_path`
        let fmc::Path(path_str) = path;
        let suffix = format!(".{}", constants::LANGUAGE_FILE_EXTENSION);
        path_str.ends_with(suffix.as_str()).and_option_from(|| {
            let cut_from = path_str.len() - suffix.len();
            Some(Location(path_str[..cut_from].to_string()))
        })
    }

    /// Obtains path (within a project context) to the file with this module.
    pub fn to_path(&self) -> file_manager_client::Path {
        // TODO [mwu] Extremely provisional. When multiple files support is
        //            added, needs to be fixed, if not earlier.
        let Location(string) = self;
        let result = format!("./{}.{}", string, constants::LANGUAGE_FILE_EXTENSION);
        file_manager_client::Path::new(result)
    }
}



// =========================
// === Module Controller ===
// =========================

shared! { Handle
    /// State data of the module controller.
    #[derive(Debug)]
    pub struct Controller {
        /// This module's location.
        location: Location,
        /// The current module used by synchronizing both module representations.
        module: Module,
        /// The id map of current ast
        // TODO: written for test purposes, should be removed once generating id_map from AST will
        // be implemented.
        id_map: IdMap,
        /// The File Manager Client handle.
        file_manager: fmc::Handle,
        /// The Parser handle
        parser: Parser,
        logger: Logger,
    }

    impl {
        /// Obtain clone of location.
        pub fn location(&self) -> Location {
            self.location.clone()
        }

        /// Updates AST after code change.
        pub fn apply_code_change(&mut self,change:&TextChangedNotification) -> FallibleResult<()> {
            let mut code        = self.code();
            let replaced_range  = change.replaced_chars.clone();
            let inserted_string = change.inserted_string();
            let replaced_size   = Size::new(replaced_range.end - replaced_range.start);
            let replaced_span   = Span::new(Index::new(replaced_range.start),replaced_size);

            code.replace_range(replaced_range,&inserted_string);
            apply_code_change_to_id_map(&mut self.id_map,&replaced_span,&inserted_string);
            self.module.ast = self.parser.parse(code, self.id_map.clone())?;
            self.logger.trace(|| format!("Applied change; Ast is now {:?}", self.module.ast));
            Ok(())
        }

        /// Read module code.
        pub fn code(&self) -> String {
            self.module.ast.repr()
        }

        /// Check if current module state is synchronized with given code. If it's not, log error,
        /// and update module state to match the `code` passed as argument.
        pub fn check_code_sync(&mut self, code:String) -> FallibleResult<()> {
            let my_code = self.code();
            if code != my_code {
                self.logger.error(|| format!("The module controller ast was not synchronized with \
                    text editor content!\n >>> Module: {:?}\n >>> Editor: {:?}",my_code,code));
                self.module.ast  = self.parser.parse(code,default())?;
                self.id_map      = default();
            }
            Ok(())
        }
    }
}

impl Handle {
    /// Create a module controller for given location.
    ///
    /// It may wait for module content, because the module must initialize its state.
    pub async fn new(location:Location, mut file_manager:fmc::Handle, mut parser:Parser)
    -> FallibleResult<Self> {
        let logger   = Logger::new(format!("Module Controller {}", location));
        logger.info(|| "Loading module file");
        let path     = location.to_path();
        file_manager.touch(path.clone()).await?;
        let content  = file_manager.read(path).await?;
        logger.info(|| "Parsing code");
        let module   = Module::try_from(parser.parse_with_metadata(content)?)?;
        logger.info(|| "Code parsed");
        logger.trace(|| format!("The parsed ast is {:?}", module.ast));
        let id_map   = module.ast.id_map();
        let data     = Controller {location,module,file_manager,parser,id_map,logger};
        Ok(Handle::new_from_data(data))
    }

    /// Save the module to file.
    pub fn save_file(&self) -> impl Future<Output=Result<(),RpcError>> {
        let (path,mut fm,code) = self.with_borrowed(|data| {
            let path = data.location.to_path();
            let fm   = data.file_manager.clone_ref();
            let code = ModuleWithMetadata::from(data.module.clone()).to_string();
            (path,fm,code)
        });
        fm.write(path.clone(),code)
    }

    #[cfg(test)]
    fn new_mock
    ( location     : Location
    , code         : &str
    , id_map       : IdMap
    , file_manager : fmc::Handle
    , mut parser   : Parser
    ) -> FallibleResult<Self> {
        let logger = Logger::new("Mocked Module Controller");
        let ast    = parser.parse(code.to_string(),id_map.clone())?;
        let module = Module {ast, metadata:default()};
        let data   = Controller {location,module,file_manager,parser,id_map,logger};
        Ok(Handle::new_from_data(data))
    }

}



#[cfg(test)]
mod test {
    use super::*;

    use ast;
    use ast::BlockLine;
    use data::text::Span;
    use data::text::TextChange;
    use data::text::TextLocation;
    use json_rpc::test_util::transport::mock::MockTransport;
    use parser::Parser;
    use uuid::Uuid;
    use wasm_bindgen_test::wasm_bindgen_test;
    use file_manager_client::Path;

    #[test]
    fn get_location_from_path() {
        let module     = Path(format!("test.{}", constants::LANGUAGE_FILE_EXTENSION));
        let not_module = Path("test.txt".to_string());

        let expected_loc = Location("test".to_string());
        assert_eq!(Some(expected_loc),Location::from_path(&module    ));
        assert_eq!(None,              Location::from_path(&not_module));
    }

    #[wasm_bindgen_test]
    fn update_ast_after_text_change() {
        let transport    = MockTransport::new();
        let file_manager = file_manager_client::Handle::new(transport);
        let parser       = Parser::new().unwrap();
        let location     = Location("Test".to_string());

        let uuid1        = Uuid::new_v4();
        let uuid2        = Uuid::new_v4();
        let module       = "2+2";
        let id_map       = IdMap(vec!
            [ (Span::from((0,1)),uuid1.clone())
            , (Span::from((2,1)),uuid2)
            ]);

        let controller   = Handle::new_mock(
            location,
            module,
            id_map,
            file_manager,
            parser
        ).unwrap();

        // Change code from "2+2" to "22+2"
        let change = TextChangedNotification {
            change        : TextChange::insert(TextLocation{line:0,column:1}, "2"),
            replaced_chars: 1..1
        };
        controller.apply_code_change(&change).unwrap();
        let expected_ast = Ast::new(ast::Module {
            lines: vec![BlockLine {
                elem: Some(Ast::new(ast::Infix {
                    larg : Ast::new(ast::Number{base:None, int:"22".to_string()}, Some(uuid1)),
                    loff : 0,
                    opr  : Ast::new(ast::Opr {name:"+".to_string()}, None),
                    roff : 0,
                    rarg : Ast::new(ast::Number{base:None, int:"2".to_string()}, Some(uuid2)),
                }, None)),
                off: 0
            }]
        }, None);
        assert_eq!(expected_ast, controller.with_borrowed(|data| data.module.ast.clone()));
    }
}
