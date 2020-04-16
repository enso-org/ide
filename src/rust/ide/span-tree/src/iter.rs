//! Module providing advanced iterators over SpanTree nodes.

use crate::Node;
use crate::node;



// ===============================
// === Chain Children Iterator ===
// ===============================

/// A stack frame of DFS searching.
#[derive(Debug)]
struct StackFrame<'a> {
    node                : &'a Node,
    child_being_visited : usize,
}

/// An iterator returned from `chain_children_iter` method of `node::Ref`. See crate's
/// documentation for more information about _chaining_.
///
/// Under the hood this iterator is performing DFS on the tree's fragment; we cut off all nodes
/// which are not root node or chained node or any child of those; then this iterator returns only
/// leaves of such subtree.
#[derive(Debug)]
pub struct ChainChildrenIterator<'a> {
    stack     : Vec<StackFrame<'a>>,
    next_node : Option<&'a node::Child>,
    base_node : node::Ref<'a>,
}

impl<'a> Iterator for ChainChildrenIterator<'a> {
    type Item = node::Ref<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_node.is_some() {
            let crumbs       = self.stack.iter().map(|sf| sf.child_being_visited);
            let return_value = self.base_node.clone().traverse_subnode(crumbs);
            self.make_dfs_step();
            self.descend_to_subtree_leaf();
            return_value
        } else {
            None
        }
    }
}

impl<'a> ChainChildrenIterator<'a> {
    /// Create iterator iterating over children of chain starting on `node`.
    pub fn new(node: node::Ref<'a>) -> Self {
        let stack     = vec![StackFrame {node:&node.node, child_being_visited:0}];
        let next_node = node.node.children.first();
        let base_node = node;
        let mut this = Self {stack,next_node,base_node};
        // Sometimes the first child is the chained node, so we must go deeper in such case.
        this.descend_to_subtree_leaf();
        this
    }

    fn make_dfs_step(&mut self) {
        if self.next_node.is_some() {
            self.next_node = None;
            while self.next_node.is_none() && !self.stack.is_empty() {
                let parent = self.stack.last_mut().unwrap();
                parent.child_being_visited += 1;
                self.next_node = parent.node.children.get(parent.child_being_visited);
                if self.next_node.is_none() {
                    self.stack.pop();
                }
            }
        }
    }

    /// For _subtree_ definition see docs for `ChainChildrenIterator`.
    fn descend_to_subtree_leaf(&mut self) {
        if let Some(mut current) = std::mem::take(&mut self.next_node) {
            while current.chained_with_parent && !current.node.children.is_empty() {
                self.stack.push(StackFrame { node: &current.node, child_being_visited: 0 });
                current = &current.node.children.first().unwrap();
            }
            self.next_node = Some(current);
        }
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use crate::builder::Builder;
    use crate::builder::TreeBuilder;



    #[test]
    fn chained_children_iterating() {
        use ast::crumbs::InfixCrumb::*;
        use ast::crumbs::PrefixCrumb::*;

        // Tree we use for tests (F means node which can be flattened):
        // root:                (-)
        //                    / |  \
        // children:        (F) ()  (F)
        //                 /|\      / | \
        // g-children:   ()()()  () () (F)
        //                   /|       / | \
        // gg-children:     ()()     ()() ()

        let tree = TreeBuilder::new(14)
            .add_ast_child(0,10,vec![LeftOperand])
                .chain_with_parent()
                .add_ast_leaf(0,3,vec![LeftOperand])
                .add_ast_leaf(4,1,vec![Operator])
                .add_ast_child(6,3,vec![RightOperand])
                    .add_ast_leaf(0,1,vec![Func])
                    .add_ast_leaf(2,1,vec![Arg])
                    .done()
                .done()
            .add_ast_leaf(11,1,vec![Operator])
            .add_ast_child(13,1,vec![RightOperand])
                .chain_with_parent()
                .add_ast_leaf(0,3,vec![LeftOperand])
                .add_ast_leaf(4,1,vec![Operator])
                .add_ast_child(6,5,vec![RightOperand])
                    .chain_with_parent()
                    .add_ast_leaf(0,1,vec![LeftOperand])
                    .add_ast_leaf(2,1,vec![Operator])
                    .add_ast_leaf(4,1,vec![RightOperand])
                    .done()
                .done()
            .build();

        let root = tree.root_ref();

        let expected_crumbs = vec!
            [ vec![0,0]
            , vec![0,1]
            , vec![0,2]
            , vec![1]
            , vec![2,0]
            , vec![2,1]
            , vec![2,2,0]
            , vec![2,2,1]
            , vec![2,2,2]
            ];
        assert_eq!(expected_crumbs, root.chain_children_iter().map(|n| n.crumbs).collect_vec());
    }
}
