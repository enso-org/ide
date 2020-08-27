//! SpanTree module
//!
//! SpanTree is a structure describing expression with nodes mapped to expression text spans. It can
//! be considered a layer over AST, that adds an information about chains (you can
//! iterate over all elements of infix chain like `1 + 2 + 3` or prefix chain like `foo bar baz`),
//! and provides interface for AST operations like set node to a new AST or add new element to
//! operator chain.

#![feature(associated_type_bounds)]
#![feature(option_result_contains)]
#![feature(trait_alias)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod action;
pub mod generate;
pub mod iter;
pub mod node;
pub mod builder;

pub use node::Node;
pub use node::Crumb;
pub use node::Crumbs;

/// Module gathering all commonly used traits for massive importing.
pub mod traits {
    pub use crate::action::Actions;
    pub use crate::generate::SpanTreeGenerator;
    pub use crate::builder::Builder;
}

/// Common types that should be visible across the whole crate.
pub mod prelude {
    pub use crate::traits::*;
    pub use ast::traits::*;
    pub use enso_prelude::*;
    pub use utils::fail::FallibleResult;
}

use traits::*;
use prelude::*;

// ==========================
// === InvocationResolver ===
// ==========================

/// Information available about some function parameter.
#[derive(Clone,Debug,Eq,PartialEq)]
#[allow(missing_docs)]
pub struct ParameterInfo {
    pub name     : Option<String>,
    pub typename : Option<String>,
    // TODO? [mwu]
    //  If needed more information could be added here, like param being suspended, defaulted, etc.

}

/// Information about a method call that span tree is concerned about.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct InvocationInfo {
    /// Information about arguments taken by a called method.
    parameters : Vec<ParameterInfo>,
}

/// Entity that is able to provide information whether a given expression is a known method
/// invocation. If so, additional information is provided.
pub trait InvocationResolver {
    /// Checks if the given expression is known to be a call to a known method. If so, returns the
    /// available information.
    fn invocation_info(&self, id:ast::Id) -> Option<InvocationInfo>;
}



// ================
// === SpanTree ===
// ================

/// A SpanTree main structure.
///
/// This structure is used to have some specific node marked as root node, to avoid confusion
/// regarding SpanTree crumbs and AST crumbs.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct SpanTree {
    /// A root node of the tree.
    pub root : Node
}

impl SpanTree {
    /// Create span tree from something that could generate it (usually AST).
    pub fn new(generator:&impl SpanTreeGenerator) -> FallibleResult<Self> {
        generator.generate_tree()
    }

    /// Get the `NodeRef` of root node.
    pub fn root_ref(&self) -> node::Ref {
        node::Ref {
            node       : &self.root,
            span_begin : default(),
            crumbs     : default(),
            ast_crumbs : default()
        }
    }

    /// Get the node (root, child, or further descendant) identified by `crumbs`.
    pub fn get_node<'a>
    (&self, crumbs:impl IntoIterator<Item=&'a Crumb>) -> FallibleResult<node::Ref> {
        self.root_ref().get_descendant(crumbs)
    }
}

impl Default for SpanTree {
    fn default() -> Self {
        let expression_id = None;
        let kind          = node::Kind::Root;
        let size          = default();
        let children      = default();
        let argument_info = default();
        let root          = Node {kind,size,children,expression_id,argument_info};
        Self {root}
    }
}
