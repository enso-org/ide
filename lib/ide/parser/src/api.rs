use crate::prelude::*;
use ast::ID;
use std::ops::Sub;

use serde::Serialize;
use serde::Deserialize;

pub type Ast = ast::Ast;


// ============
// == Parser ==
// ============

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Index { pub value:usize }

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Size { pub value:usize }

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Span { pub index: Index, pub size: Size }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IDMap(pub Vec<(Span, ID)>);

impl Add for Size {
    type Output = Size;
    fn add(self, rhs:Size) -> Size {
        Size { value: self.value + rhs.value }
    }
}

impl Add<Size> for Index {
    type Output = Index;
    fn add(self, rhs:Size) -> Index {
        Index { value: self.value + rhs.value }
    }
}

impl Sub<Size> for Index {
    type Output = Index;
    fn sub(self, rhs:Size) -> Index {
        Index { value: self.value - rhs.value }
    }
}


/// Entity being able to parse Luna programs into Luna's AST.
pub trait IsParser {
    fn parse(&mut self, program: String, ids: IDMap) -> Result<Ast>;
}


// ===========
// == Error ==
// ===========

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    /// Error due to inner workings of the parser.
    #[fail(display = "Internal parser error: {:?}", _0)]
    ParsingError(String),
    /// Error related to wrapping = communication with the parser service.
    #[fail(display = "Interop error: {}", _0)]
    InteropError(#[cause] Box<dyn failure::Fail>),
}

/// Wraps an arbitrary `std::error::Error` as an `InteropError.`
pub fn interop_error<T>(error: T) -> Error
    where T: Fail {
    Error::InteropError(Box::new(error))
}
