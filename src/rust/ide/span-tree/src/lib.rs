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
    pub use ast::traits::*;
    pub use enso_prelude::*;
    pub use utils::fail::FallibleResult;
}

use prelude::*;


// ==============
// === Crumbs ===
// ==============

/// A possible connection endpoint described by span tree crumbs and ast crumbs.
///
/// In case that endpoint is the span tree node, ast crumbs are empty. Otherwise, they are relative
/// to the AST corresponding to the span tree node.
#[derive(Clone,Debug,Default,PartialEq,PartialOrd)]
pub struct SplitCrumbs {
    /// Crumbs to a Span Tree leaf.
    pub head : Vec<usize>,
    /// Crumbs for traversing AST corresponding to the span tree leaf.
    /// Might be empty, if the span tree node corresponds to the desired AST node.
    pub tail : ast::Crumbs,
}

impl SplitCrumbs {
    pub fn new
    (span_crumbs:impl IntoIterator<Item=usize>, ast_crumbs:impl ast::crumbs::IntoCrumbs)
     -> SplitCrumbs {
        SplitCrumbs {
            head : span_crumbs.into_iter().collect(),
            tail : ast_crumbs.into_crumbs(),
        }
    }

    pub fn new_span(span_crumbs:impl IntoIterator<Item=usize>) -> SplitCrumbs {
        SplitCrumbs {
            head : span_crumbs.into_iter().collect(),
            tail : default(),
        }
    }
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

    /// Converts `Ast` crumbs to `SpanTree` crumbs.
    ///
    /// Interestingly, this never fails. At worst none of the Ast crumbs will be matched and all
    /// will be placed in the `tail` part of or the `SplitCrumbs`.
    pub fn convert_from_ast_crumbs(&self, ast_crumbs:&[ast::Crumb]) -> SplitCrumbs {
        self.root.convert_from_ast_crumbs(ast_crumbs)
    }

    /// Converts `SpanTree` crumbs to `Ast` crumbs.
    pub fn convert_to_ast_crumbs(&self, crumbs:SplitCrumbs) -> Option<ast::Crumbs> {
        let root = self.root_ref();
        let node_ref = root.traverse_subnode(crumbs.head.iter().copied())?;
        let mut ret = node_ref.ast_crumbs;
        ret.extend(crumbs.tail);
        Some(ret)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::crumbs;

    #[test]
    fn crumb_conversion_test() {
        let code = "1 + 2 + 3";
        let ast  = parser::Parser::new_or_panic().parse_line(code).unwrap();
        let tree = SpanTree::new(&ast).unwrap();

        use ast::crumbs::InfixCrumb::*;
        use ast::crumbs::PrefixCrumb::*;

        let test_conversions0 = |ast_crumbs:ast::Crumbs, crumbs:SplitCrumbs| {
            assert_eq!(tree.convert_from_ast_crumbs(&ast_crumbs), crumbs);
            assert_eq!(tree.convert_to_ast_crumbs(crumbs).as_ref(), Some(&ast_crumbs));
        };

        // Tester to be used when the crumb refers to span tree node.
        let expect_node_match = |ast_crumbs:ast::Crumbs, crumbs:&[usize]| {
            let split_crumbs = SplitCrumbs::new_span(crumbs.iter().copied());
            test_conversions0(ast_crumbs,split_crumbs)
        };

        let sub_node_match = |ast_crumbs:ast::Crumbs, head:&[usize], tail:ast::Crumbs| {
            let split_crumbs = SplitCrumbs::new(head.iter().copied(),tail);
            test_conversions0(ast_crumbs,split_crumbs)
        };

        let expect_node_mismatch = |ast_crumbs:ast::Crumbs| {
            let split_crumbs = SplitCrumbs::new(vec![],ast_crumbs.iter().cloned());
            test_conversions0(ast_crumbs,split_crumbs)
        };

        expect_node_match(crumbs![],                         &[]   );
        expect_node_match(crumbs![LeftOperand],              &[0]  );
        expect_node_match(crumbs![Operator],                 &[1]  );
        expect_node_match(crumbs![RightOperand],             &[2]  );
        expect_node_match(crumbs![LeftOperand,LeftOperand],  &[0,0]);
        expect_node_match(crumbs![LeftOperand,Operator],     &[0,1]);
        expect_node_match(crumbs![LeftOperand,RightOperand], &[0,2]);

        expect_node_mismatch(crumbs![Arg]);
        // expect_node_mismatch(crumbs![LeftOperand,Arg]         );
        // expect_node_mismatch(crumbs![RightOperand,LeftOperand]);
        //
        // expect_node_mismatch(crumbs![Arg]                     );
        // expect_node_mismatch(crumbs![LeftOperand,Arg]         );
        // expect_node_mismatch(crumbs![RightOperand,LeftOperand]);

        // assert!(tree.convert_to_ast_crumbs(vec![1,5]).is_none());
        // assert!(tree.convert_to_ast_crumbs(vec![0,0,0]).is_none());
    }
}
