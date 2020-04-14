use crate::prelude::*;

use crate::iter::{DfsIterator, DfsMode};

use data::text::Index;
use data::text::Size;



// =============
// === Nodes ===
// =============

/// A type of SpanTree node.
#[derive(Debug)]
pub enum NodeType {
    /// The root node covering the whole expression on which the span tree was generated.
    Root,
    /// The non-root node which have corresponding AST node.
    AstChild {crumbs_from_parent:ast::Crumbs},
    /// An empty node being a placeholder for adding new children to the parent. The empty node
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
/// SpanTree nodes can be _chained_. Sometimes we traverse SpanTree over less granular
/// representation, where we treat whole chains of nodes as a one node (but it does not affect the
/// crumbs identifying nodes)
///
/// The reason we do such chainging is to make simpler representation for expressions
/// like `1 + 2 + 3 + 4`. This expression have one chain of all nodes corresponding to Infixes, and
/// this single chain will have children representing `['1', '+', '2', '+', '3', '+', '4']` - that
/// is much simpler than 4-level high tree.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct Node {
    /// An offset counted from the parent node starting index to the start of this node's span.
    pub offset              : Size,
    pub len                 : Size,
    pub node_type           : NodeType,
    pub children            : Vec<Node>,
    pub chained_with_parent : bool,
}

impl Node {
    /// Create new empty node.
    pub fn new_empty(offset:Size) -> Self {
        let node_type      = NodeType::Empty;
        let len            = Size::new(0);
        let children       = Vec::new();
        let can_be_chained = false;
        Node {node_type, offset,len,children, chained_with_parent: can_be_chained }
    }
}


// === Node Reference ===

/// A reference to node in SpanTree with information about both SpanTree crumbs identifying this
/// node as well as optionally the AST crumbs pointing to the corresponding AST node.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub struct NodeRef<'a> {
    pub node              : &'a Node,
    pub crumbs            : Vec<usize>,
    pub parent_ast_crumbs : ast::Crumbs,
    pub span_index        : Index,
}

impl<'a> NodeRef<'a> {

    /// Get the AST crumbs which points to the corresponding AST node, or None, if there is no such
    /// AST node.
    pub fn ast_crumbs(&self) -> Option<ast::Crumbs> {
        match &self.node.node_type {
            NodeType::Root                         => Some(default()),
            NodeType::Empty                        => None,
            NodeType::AstChild{crumbs_from_parent} => {
                let mut crumbs = self.parent_ast_crumbs.clone();
                crumbs.extend(crumbs_from_parent.iter());
                Some(crumbs)
            },
        }
    }

    /// Get the reference to child with given index. Returns None if index if out of bounds.
    pub fn child(mut self, index:usize) -> Option<NodeRef<'a>> {
        self.node.children.get(index).map(|child| {
            if let NodeType::AstChild{crumbs_from_parent} = &self.node.node_type {
                self.parent_ast_crumbs.extend(crumbs_from_parent);
            }
            self.crumbs.push(index);
            self.span_index += child.offset;
            self.node = child;
            self
        })
    }

    /// DFS Iterate over whole sub-tree starting from this node.
    pub fn dfs_iter(self) -> impl Iterator<Item=NodeRef<'a>> {
        DfsIterator::new(self,DfsMode::All)
    }

    /// DFS Iterate over whole sub-tree starting from this node, and chaining potential
    pub fn chains_dfs_iter(self) -> impl Iterator<Item=NodeRef<'a>> {
        DfsIterator::new(self,DfsMode::AllFlatten)
    }

    /// Iterate over all children of chain starting from this node.
    pub fn chain_children_iter(self) -> impl Iterator<Item=NodeRef<'a>> {
        DfsIterator::new(self,DfsMode::OneLevelFlatten).skip(1)
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
        let node              = &self.root;
        let crumbs            = default();
        let parent_ast_crumbs = default();
        let span_index        = Index::default() + node.offset;
        NodeRef {node,crumbs,parent_ast_crumbs,span_index}
    }
}



// ============
// === Test ===
// ============

