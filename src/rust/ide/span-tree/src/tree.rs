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
    /// The non-root node which have corresponding AST node.
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
///
/// SpanTree nodes can be _chained_, which allows to iterate over all children of single chain,
/// so it is possible to simplify representation for expressions like `1 + 2 + 3 + 4`. This
/// expression have one chain of all nodes corresponding to Infixes, and
/// this single chain will have children representing `['1', '+', '2', '+', '3', '+', '4']` - that
/// is much simpler than 4-level high tree.
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

#[derive(Debug,Eq,PartialEq)]
pub struct Child {
    pub node                : Node,
    /// An offset counted from the parent node starting index to the start of this node's span.
    pub offset              : Size,
    pub chained_with_parent : bool,
    pub ast_crumbs          : ast::Crumbs,
}



// === Node Reference ===

/// A reference to node in SpanTree with information about both SpanTree crumbs identifying this
/// node as well as optionally the AST crumbs pointing to the corresponding AST node.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct NodeRef<'a> {
    pub node       : &'a Node,
    pub span_begin : Index,
    pub crumbs     : Vec<usize>,
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

    /// Iterate over all children of this and all chained nodes.
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

#[derive(Debug)]
pub struct SpanTree {
    pub root : Node
}

impl SpanTree {

    /// Get the node identified by crumbs.
    pub fn get_node(&self, crumbs:impl Crumbs) -> Option<NodeRef> {
        self.root_ref().traverse_subnode(crumbs)
    }

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

