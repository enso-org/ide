//! Module Controller.
//!
//! The module controller keeps cached module state (module state is AST+Metadata or equivalent),
//! and uses it for synchronizing state for text and graph representations. It provides method
//! for registering text and graph changes. If for example text represntation will be changed, there
//! will be notifications for both text change and graph change.
//!
//! This module is still on WIP state, for now it contains stubs only.

use crate::prelude::*;

use crate::controller::FallibleResult;
use crate::double_representation::apply_code_change_to_id_map;

use ast::Ast;
use ast::HasRepr;
use ast::IdMap;
use data::text::Index;
use data::text::Size;
use data::text::Span;
use file_manager_client as fmc;
use parser::SharedParser;
use shapely::shared;
use parser::api::IsParser;
use basegl::display::shape::text::text_field::TextChangedNotification;


// =======================
// === Module Location ===
// =======================

/// Structure uniquely identifying module location in the project.
/// Mappable to filesystem path.
#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct Location(pub String);

impl Location {
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
        /// The current module ast, used by synchronizing both module representations.
        ast: Ast,
        /// The id map of current ast
        // TODO: written for test purposes, should be removed once generating id_map from AST will
        // be implemented.
        id_map: IdMap,
        /// The File Manager Client handle.
        file_manager: fmc::Handle,
        /// The Parser handle
        parser: SharedParser,
    }

    impl {
        /// Obtain clone of location.
        pub fn location(&self) -> Location {
            self.location.clone()
        }

        /// Updates AST after code change.
        pub fn apply_code_change(&mut self,change:TextChangedNotification) {
            let mut code        = self.ast.repr();
            let replaced_range  = change.replaced_range_char;
            let inserted_string = change.inserted_string.as_str();
            let replaced_size   = Size::new(replaced_range.end - replaced_range.start);
            let replaced_span   = Span::new(Index::new(replaced_range.start),replaced_size);
            code.replace_range(replaced_range,inserted_string);
            apply_code_change_to_id_map(&mut self.id_map,&replaced_span,inserted_string);
            self.ast = self.parser.parse(code, self.id_map.clone()).unwrap();
        }
    }
}

impl Handle {
    pub async fn new(location:Location, mut file_manager:fmc::Handle, mut parser:SharedParser)
    -> FallibleResult<Self> {
        let path    = location.to_path();
        let content = file_manager.read(path).await?;
        let ast     = parser.parse(content,default())?;
        let id_map  = default();
        let data    = Controller {location,ast,file_manager,parser,id_map};
        Ok(Handle::new_from_data(data))
    }

    #[cfg(test)]
    fn new_mock
    (location:Location, code:&str, id_map:IdMap, file_manager:fmc::Handle, mut parser:SharedParser)
    -> FallibleResult<Self> {
        let ast      = parser.parse(code.to_string(),id_map.clone())?;
        let data     = Controller {location,ast,file_manager,parser,id_map};
        Ok(Handle::new_from_data(data))
    }
}



#[cfg(test)]
mod test {

}
