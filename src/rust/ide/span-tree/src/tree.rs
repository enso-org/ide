//! A module with SpanTree structure definition.

use crate::prelude::*;

use crate::iter::ChainChildrenIterator;

use data::text::Index;
use data::text::Size;



// =============
// === Nodes ===
// =============

/// A type of SpanTree node.
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Type {
    /// The node which have corresponding AST node.
    Ast,
    /// An empty node being a placeholder for adding new child to the parent. The empty node
    /// should not have any further children.
    Empty
}

/// A type which identifies some node in SpanTree. This is essentially a iterator over child
/// indices, so `[4]` means _root's fifth child_, `[4, 2]`means _the third child of root's fifth
/// child_ and so on.
pub trait Crumbs = IntoIterator<Item=usize>;


// === Node ===

/// SpanTree Node.
///
/// Each node in SpanTree is bound to some span of code, and potentially may have corresponding
/// AST node.
#[derive(Debug,Eq,PartialEq)]
#[allow(missing_docs)]
pub struct Node {
    pub node_type : Type,
    pub len       : Size,
    pub children  : Vec<Child>,
}

impl Node {
    /// Create new empty node.
    pub fn new_empty() -> Self {
        let node_type           = Type::Empty;
        let len                 = Size::new(0);
        let children            = Vec::new();
        Node {node_type,len,children}
    }
}

/// A structure which contains `Node` being a child of some parent. It contains some additional
/// data regarding this relation
#[derive(Debug,Eq,PartialEq)]
pub struct Child {
    /// A child node.
    pub node                : Node,
    /// An offset counted from the parent node starting index to the start of this node's span.
    pub offset              : Size,
    /// Flag indicating that parent should take this node's children instead of itself when
    /// iterating using `chain_children_iter` method. See this method docs for reference, and
    /// crate's doc for details about _chaining_.
    pub chained_with_parent : bool,
    /// AST crumbs which lead from parent to child associated AST node.
    pub ast_crumbs          : ast::Crumbs,
}



// === Node Reference ===

/// A reference to node inside some specific tree.
#[derive(Clone,Debug)]
pub struct NodeRef<'a> {
    /// The node's ref.
    pub node       : &'a Node,
    /// Span begin being an index counted from the root expression.
    pub span_begin : Index,
    /// Crumbs specifying this node position related to root. See `Crumbs` docs.
    pub crumbs     : Vec<usize>,
    /// Ast crumbs locating associated AST node, related to the root's AST node.
    pub ast_crumbs : ast::Crumbs,
}

impl<'a> NodeRef<'a> {

    /// Get the reference to child with given index. Returns None if index if out of bounds.
    pub fn child(mut self, index:usize) -> Option<NodeRef<'a>> {
        self.node.children.get(index).map(|child| {
            self.crumbs.push(index);
            self.ast_crumbs.extend(&child.ast_crumbs);
            self.span_begin += child.offset;
            self.node = &child.node;
            self
        })
    }

    /// Iterate over all children of operator/prefix chain starting from this node. See crate's
    /// documentation for more information about _chaining_.
    pub fn chain_children_iter(self) -> impl Iterator<Item=NodeRef<'a>> {
        ChainChildrenIterator::new(self)
    }

    /// Get the sub-node (child, or further descendant) identified by `crumbs`.
    pub fn traverse_subnode(self, crumbs:impl Crumbs) -> Option<NodeRef<'a>> {
        let mut iter = crumbs.into_iter();
        match iter.next() {
            Some(index) => self.child(index).and_then(|child| child.traverse_subnode(iter)),
            None        => Some(self)
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
#[derive(Debug,Eq,PartialEq)]
pub struct SpanTree {
    pub root : Node
}

impl SpanTree {
    /// Get the `NodeRef` of root node.
    pub fn root_ref(&self) -> NodeRef {
        NodeRef {
            node: &self.root,
            span_begin : default(),
            crumbs     : default(),
            ast_crumbs : default()
        }
    }
}



// ============
// === Test ===
// ============

#[cfg(test)]
mod test {
    use super::*;
    use crate::builder::Builder;
    use crate::builder::RootBuilder;
    use ast::crumbs::InfixCrumb;

    #[test]
    fn traversing_tree() {
        let tree = RootBuilder::new(7)
            .add_empty_child(0)
            .add_ast_leaf(0,1,vec![InfixCrumb::LeftOperand])
            .add_ast_leaf(1,1,vec![InfixCrumb::Operator])
            .add_ast_child(2,5,vec![InfixCrumb::RightOperand])
                .add_ast_leaf(0,2,vec![InfixCrumb::LeftOperand])
                .add_ast_leaf(3,1,vec![InfixCrumb::Operator])
                .add_ast_leaf(4,1,vec![InfixCrumb::RightOperand])
                .done()
            .build();

        let root         = tree.root_ref();
        let child1       = root.clone().traverse_subnode(vec![0]).unwrap();
        let child2       = root.clone().traverse_subnode(vec![2]).unwrap();
        let grand_child1 = root.clone().traverse_subnode(vec![2,0]).unwrap();
        let grand_child2 = child2.clone().traverse_subnode(vec![2,1]).unwrap();

        // Span begin.
        assert_eq!(0, root.span_begin.value);
        assert_eq!(0, child1.span_begin.value);
        assert_eq!(2, child2.span_begin.value);
        assert_eq!(2, grand_child1.span_begin.value);
        assert_eq!(5, grand_child2.span_begin.value);

        // Length
        assert_eq!(7, root.len.value);
        assert_eq!(1, child1.len.value);
        assert_eq!(5, child2.len.value);
        assert_eq!(2, grand_child1.len.value);
        assert_eq!(1, grand_child2.len.value);

        // crumbs
        assert_eq!(vec![]   , root.crumbs);
        assert_eq!(vec![0]  , child1.crumbs);
        assert_eq!(vec![2]  , child2.crumbs);
        assert_eq!(vec![2,0], grand_child1.crumbs);
        assert_eq!(vec![2,1], grand_child2.crumbs);

        // AST crumbs
        assert_eq!(vec![]                                                , root.ast_crumbs);
        assert_eq!(vec![InfixCrumb::LeftOperand]                         , child1.ast_crumbs);
        assert_eq!(vec![InfixCrumb::RightOperand]                        , child2.ast_crumbs);
        assert_eq!(vec![InfixCrumb::RightOperand,InfixCrumb::LeftOperand], grand_child1.ast_crumbs);
        assert_eq!(vec![InfixCrumb::RightOperand,InfixCrumb::Operator]   , grand_child2.ast_crumbs);

        // Not existing nodes

        assert_eq!(None, root.traverse_subnode(vec![3]));
        assert_eq!(None, root.traverse_subnode(vec![1,0]));
        assert_eq!(None, root.traverse_subnode(vec![2,1,0]));
        assert_eq!(None, root.traverse_subnode(vec![2,5]));
        assert_eq!(None, root.traverse_subnode(vec![2,5,0]));
    }
}
