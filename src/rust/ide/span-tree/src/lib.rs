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
#[cfg(test)]
pub mod builder;

pub use node::Node;

/// Module gathering all commonly used traits for massive importing.
pub mod traits {
    pub use crate::action::Actions;
    pub use crate::generate::SpanTreeGenerator;
    #[cfg(test)]
    pub use crate::builder::Builder;
}

/// Common types that should be visible across the whole crate.
pub mod prelude {
    pub use crate::traits::*;
    pub use enso_prelude::*;
    pub use utils::fail::FallibleResult;
}

use prelude::*;



// ==============
// === Crumbs ===
// ==============

/// Crumb identifies subtree within a node. It is the index of the child node.
pub type Crumb = usize;

/// Crumbs identifying node's location in the span tree.
pub type Crumbs = Vec<Crumb>;



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

    /// Converts `Ast` crumbs to `SpanTree` crumbs.
    pub fn convert_from_ast_crumbs(&self, ast_crumbs:&[ast::Crumb]) -> Option<Vec<usize>> {
        self.root.convert_from_ast_crumbs(ast_crumbs)
    }

    pub fn convert_to_ast_crumbs(&self, crumbs:&[Crumb]) -> Option<ast::Crumbs> {
        self.root.convert_to_ast_crumbs(crumbs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::crumbs;

    #[test]
    fn ast_crumbs_to_span_tree_crumbs() {
        let code = "1 + 2 + 3";
        let ast  = parser::Parser::new_or_panic().parse_line(code).unwrap();
        let tree = SpanTree::new(&ast).unwrap();

        use ast::crumbs::InfixCrumb::*;
        use ast::crumbs::PrefixCrumb::*;

        let test_conversions = |ast_crumbs:ast::Crumbs, crumbs:Crumbs| {
            assert_eq!(tree.convert_from_ast_crumbs(&ast_crumbs).as_ref(), Some(&crumbs));
            assert_eq!(tree.convert_to_ast_crumbs(&crumbs).as_ref(),       Some(&ast_crumbs));
        };

        test_conversions(crumbs![],                         vec![]   );
        test_conversions(crumbs![LeftOperand],              vec![0]  );
        test_conversions(crumbs![Operator],                 vec![1]  );
        test_conversions(crumbs![RightOperand],             vec![2]  );
        test_conversions(crumbs![LeftOperand,LeftOperand],  vec![0,0]);
        test_conversions(crumbs![LeftOperand,Operator],     vec![0,1]);
        test_conversions(crumbs![LeftOperand,RightOperand], vec![0,2]);

        assert!(tree.convert_from_ast_crumbs(&crumbs![Arg]).is_none());
        assert!(tree.convert_from_ast_crumbs(&crumbs![LeftOperand,Arg]).is_none());
        assert!(tree.convert_from_ast_crumbs(&crumbs![RightOperand,LeftOperand]).is_none());

        assert!(tree.convert_to_ast_crumbs(&vec![4]).is_none());
        assert!(tree.convert_to_ast_crumbs(&vec![1,5]).is_none());
        assert!(tree.convert_to_ast_crumbs(&vec![0,0,0]).is_none());
    }
}
