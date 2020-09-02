//! Span tree shape and contained data depends not only on the AST but also some context-dependent
//! information. This module defined trait [`Context`] that provides the information known to
//! Span Tree during its construction.

use crate::prelude::*;

use crate::InvocationInfo;



// ===============
// === Context ===
// ===============

/// Entity that is able to provide information whether a given expression is a known method
/// invocation. If so, additional information is provided.
pub trait Context {
    /// Checks if the given expression is known to be a call to a known method. If so, returns the
    /// available information.
    fn invocation_info(&self, id:ast::Id) -> Option<InvocationInfo>;

    /// Build a new context that merges this context and the one given in argument that will be used
    /// as a fallback.
    fn merge<U>(self, other:U) -> Merged<Self,U>
        where Self:Sized, U:Context {
        Merged::new(self,other)
    }
}

fn a(_:Box<dyn Context>) {} // TODO remove



// ===============
// === Context ===
// ===============

/// Represents a context created from merging two other contexts.
#[derive(Clone,Debug)]
pub struct Merged<First,Second> {
    first  : First,
    second : Second
}

impl<First,Second> Merged<First,Second> {
    /// Creates a context merging the contexts from arguments.
    pub fn new(first:First, second:Second) -> Self {
        Self {
            first,second
        }
    }
}

impl<First,Second> Context for Merged<First,Second>
    where First  : Context,
          Second : Context {
    fn invocation_info(&self, id:ast::Id) -> Option<InvocationInfo> {
        self.first.invocation_info(id).or_else(|| self.second.invocation_info(id))
    }
}



// =============
// === Empty ===
// =============

/// An empty context that provides no information whatsoever.
#[derive(Copy,Clone,Debug)]
pub struct Empty;

impl Context for Empty {
    fn invocation_info(&self, _id:ast::Id) -> Option<InvocationInfo> {
        None
    }
}

