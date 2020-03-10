//! A module containing structures and traits used in parser API.

use crate::prelude::*;

use ast::IdMap;
use ast::HasRepr;
use ast::HasIdMap;

pub use ast::Ast;

use serde::Deserialize;
use serde::Serialize;


// ============
// == Module ==
// ============

/// Parsed file / module with metadata
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ModuleWithMetadata {
    /// ast representation
    pub ast: Ast,
    /// raw metadata in json
    pub metadata: serde_json::Value
}

const ID_TAG       : &str = "# [idmap] ";
const METADATA_TAG : &str = "# [metadata] ";

impl ToString for ModuleWithMetadata {
    fn to_string(&self) -> String {
        let remove_newlines = |string:String| string
            .chars()
            .filter(|c| c != &'\n' && c != &'\r')
            .collect::<String>();

        let code = self.ast.repr();
        let ids  = remove_newlines(
            serde_json::to_string(&self.ast.id_map()).expect(
                "It should be possible to serialize idmap."
            )
        );
        let meta = remove_newlines(
            serde_json::to_string(&self.metadata).expect(
                "It should be possible to serialize metadata."
            )
        );
        format!("{}\n\n\n{}{}\n{}{}", code, ID_TAG, ids, METADATA_TAG, meta)
    }
}


// ============
// == Parser ==
// ============

/// Entity being able to parse programs into AST.
pub trait IsParser : Debug {
    /// Parse program.
    fn parse(&mut self, program:String, ids:IdMap) -> Result<Ast>;

    /// Parse a module content that contains idmap and metadata.
    fn parse_with_metadata
    (&mut self, program:String) -> Result<ModuleWithMetadata>;
}



// ===========
// == Error ==
// ===========

/// A result of parsing code.
pub type Result<T> = std::result::Result<T, Error>;

/// An error which may be result of parsing code.
#[derive(Debug, Fail)]
pub enum Error {
    /// Error due to inner workings of the parser.
    #[fail(display = "Internal parser error: {:?}", _0)]
    ParsingError(String),
    /// Error related to wrapping = communication with the parser service.
    #[fail(display = "Interop error: {}", _0)]
    InteropError(#[cause] Box<dyn Fail>),
}

/// Wraps an arbitrary `std::error::Error` as an `InteropError.`
pub fn interop_error<T>(error:T) -> Error
    where T: Fail {
    Error::InteropError(Box::new(error))
}
