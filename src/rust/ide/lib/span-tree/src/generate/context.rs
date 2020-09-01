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

    fn merge<U>(self, other:U) -> Merged<Self,U>
        where Self:Sized, U:Context {
        Merged::new(self,other)
    }
}

fn a(_:Box<dyn Context>) {} // TODO remove



// ===============
// === Context ===
// ===============

#[derive(Clone,Debug)]
pub struct Merged<First,Second> {
    first  : First,
    second : Second
}

impl<First,Second> Merged<First,Second> {
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

#[derive(Copy,Clone,Debug)]
pub struct Empty;

impl Context for Empty {
    fn invocation_info(&self, _id:ast::Id) -> Option<InvocationInfo> {
        None
    }
}

