//! A module containing structures and traits used in parser API.

use crate::prelude::*;

use ast::IdMap;
use ast::HasRepr;
use ast::HasIdMap;

pub use ast::Ast;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;



// ================
// == SourceFile ==
// ================

/// Things that are metadata.
pub trait Metadata:Serialize+DeserializeOwned {}

/// Parsed file / module with metadata.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SourceFile<M:Metadata> {
    /// Ast representation.
    pub ast: Ast,
    /// Raw metadata in json.
    #[serde(bound(deserialize = ""))]
    pub metadata: M
}

const ID_TAG       : &str = "# [idmap] ";
const METADATA_TAG : &str = "# [metadata] ";

fn to_json_single_line
(val:&impl Serialize) -> std::result::Result<String,serde_json::Error> {
    let json = serde_json::to_string(val)?;
    let line = json.chars().filter(|c| c != &'\n' && c != &'\r').collect();
    Ok(line)
}

impl<M:Metadata> ToString for SourceFile<M> {
    fn to_string(&self) -> String {
        let code = self.ast.repr();
        let ids  = to_json_single_line(&self.ast.id_map()).expect(
            "It should be possible to serialize idmap."
        );
        let meta = to_json_single_line(&self.metadata).expect(
            "It should be possible to serialize metadata."
        );
        iformat!("{code}\n\n\n{ID_TAG}{ids}\n{METADATA_TAG}{meta}")
    }
}


// ============
// == Parser ==
// ============

/// Entity being able to parse programs into AST.
pub trait IsParser : Debug {
    /// Parse program.
    fn parse(&mut self, program:String, ids:IdMap) -> Result<Ast>;

    /// Parse contents of the program source file,
    /// where program code may be followed by idmap and metadata.
    fn parse_with_metadata<M:Metadata>
    (&mut self, program:String) -> Result<SourceFile<M>>;
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
