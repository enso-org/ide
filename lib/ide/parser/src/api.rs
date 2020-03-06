//! A module containing structures and traits used in parser API.

use crate::prelude::*;

use ast::{IdMap, IdMetadataMap};

pub use ast::Ast;


// ============
// == Parser ==
// ============

const METATAG:&str = "# [metadata]";
const IDTAG:&str = "# [idmap]";

/// Entity being able to parse programs into AST.
pub trait IsParser : Debug {
    /// Parse program.
    fn parse(&mut self, program:String, ids:IdMap) -> Result<Ast>;

    fn parse_file(&mut self, file:String) -> Result<(Ast,IdMetadataMap)> {
        let lines = &file.lines().rev().take(2).collect_vec()[..];
        if lines[0].starts_with(METATAG) {
            let meta = serde_json::from_str(lines[0].trim_start_matches(METATAG))?;
            let ids  = serde_json::from_str(lines[1].trim_start_matches(IDTAG))?;
            let code = &file.lines().rev().skip(2).rev().collect();
            let ast  = self.parse(code, ids)?;
            Ok((ast, meta))
        }
        else {
            let ast  = self.parse(file,default())?;
            Ok((ast, default()))
        }
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
