//! A module containing structures and traits used in parser API.

use crate::prelude::*;

use ast::HasRepr;
use ast::HasIdMap;
use data::text::Index;

pub use ast::Ast;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;



// ================
// == SourceFile ==
// ================

/// Things that are metadata.
pub trait Metadata:Serialize+DeserializeOwned {}

/// Raw metadata.
impl Metadata for serde_json::Value {}

/// Parsed file / module with metadata.
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub struct SourceFile<Metadata> {
    /// Ast representation.
    pub ast: ast::known::Module,
    /// Raw metadata in json.
    pub metadata: Metadata
}

pub struct SerializedSourceFile {
    pub string   : String,
    pub code     : Range<Index>,
    pub id_map   : Range<Index>,
    pub metadata : Range<Index>,
}

const METADATA_TAG:&str = "\n\n\n#### METADATA ####\n";

fn to_json_single_line(val:&impl Serialize) -> std::result::Result<String,serde_json::Error> {
    let json = serde_json::to_string(val)?;
    let line = json.chars().filter(|c| *c != '\n' && *c != '\r').collect();
    Ok(line)
}

impl<M:Metadata> SourceFile<M> {
    pub fn serialize_(&self) -> std::result::Result<SerializedSourceFile,serde_json::Error> {
        let code                  = self.ast.repr();
        let id_map                = to_json_single_line(&self.ast.id_map())?;
        let metadata              = to_json_single_line(&self.metadata)?;
        let id_map_start          = code.len() + METADATA_TAG.len();
        let newlines_after_id_map = 1;
        let metadata_start        = id_map_start + id_map.len() + newlines_after_id_map;
        Ok(SerializedSourceFile {
            string   : iformat!("{code}{METADATA_TAG}{id_map}\n{metadata}"),
            code     : Index::new(0)             ..Index::new(code.len()),
            id_map   : Index::new(id_map_start)  ..Index::new(id_map_start + id_map.len()),
            metadata : Index::new(metadata_start)..Index::new(metadata_start + metadata.len()),
        })
    }
}

impl<M:Metadata> TryFrom<&SourceFile<M>> for String {
    type Error = serde_json::Error;
    fn try_from(val:&SourceFile<M>) -> std::result::Result<String,Self::Error> {
        Ok(val.serialize_()?.string)
    }
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
    #[fail(display = "Internal parser error: {:?}.", _0)]
    ParsingError(String),
    /// Parser returned non-module AST root.
    #[fail(display = "Internal parser error: non-module root node.")]
    NonModuleRoot,
    /// Error related to wrapping = communication with the parser service.
    #[fail(display = "Interop error: {}.", _0)]
    InteropError(#[cause] Box<dyn Fail>),
}

/// When trying to parse a line, not a single line was produced.
#[derive(Debug,Fail,Clone,Copy)]
#[fail(display = "Expected a single line, parsed none.")]
pub struct NoLinesProduced;

/// When trying to parse a single line, more were generated.
#[derive(Debug,Fail,Clone,Copy)]
#[fail(display = "Expected just a single line, found more.")]
pub struct TooManyLinesProduced;

/// Wraps an arbitrary `std::error::Error` as an `InteropError.`
pub fn interop_error<T>(error:T) -> Error
    where T: Fail {
    Error::InteropError(Box::new(error))
}



// =============
// === Tests ===
// =============

mod test {
    use super::*;

    use std::str::FromStr;
    use uuid::Uuid;


    #[derive(Clone,Debug,Deserialize,Serialize)]
    struct Metadata {
        foo : usize,
    }

    impl crate::api::Metadata for Metadata {}

    #[test]
    fn serializing_source_file() {
        let node_id  = Uuid::from_str("89f29e9f-cd21-4d35-93ce-d6df9333e2cd").unwrap();
        let main     = ast::Ast::var("main");
        let node     = ast::Ast::infix_var("2","+","2").with_id(node_id);
        let infix    = ast::Ast::infix(main,"=",node);
        let ast      = ast::Ast::one_line_module(infix).try_into().unwrap();
        let metadata = Metadata{foo:321};
        let source   = SourceFile {ast,metadata};

        let serialized = source.serialize_().unwrap();
        let expected   = r#"main = 2 + 2


#### METADATA ####
[[{"index":{"value":7},"size":{"value":5}},"89f29e9f-cd21-4d35-93ce-d6df9333e2cd"]]
{"foo":321}"#;
        assert_eq!(serialized.string  , expected.to_string());
        assert_eq!(serialized.code    , Index::new(0)  ..Index::new(12));
        assert_eq!(serialized.id_map  , Index::new(34) ..Index::new(117));
        assert_eq!(serialized.metadata, Index::new(118)..Index::new(129));
    }
}