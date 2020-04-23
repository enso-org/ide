//! Module providing advanced iterators over SpanTree nodes.
use crate::prelude::*;

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

/// Defines a subtree of SpanTree we're iterating over.
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum TreeFragment {
    /// The whole SpanTree
    AllNodes,
    /// Only the searching root, chained with root and their children.
    ChainAndDirectChildren
}

/// An iterator over the leafs of some specific fragment of SpanTree. See `TreeFragment` for
/// supported _fragment_ kinds.
#[derive(Debug)]
pub struct LeafIterator<'a> {
    stack     : Vec<StackFrame<'a>>,
    next_node : Option<&'a node::Child>,
    base_node : node::Ref<'a>,
    fragment  : TreeFragment,
}

impl<'a> Iterator for LeafIterator<'a> {
    type Item = node::Ref<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_node.is_some() {
            let crumbs       = self.stack.iter().map(|sf| sf.child_being_visited);
            let return_value = self.base_node.clone().traverse_subnode(crumbs);
            self.make_dfs_step();
            self.descend_to_leaf();
            return_value
        } else {
            None
        }
    }
}

impl<'a> LeafIterator<'a> {
    /// Create iterator iterating over leafs of subtree rooted  on `node`.
    pub fn new(node: node::Ref<'a>, fragment:TreeFragment) -> Self {
        let stack     = vec![StackFrame {node:&node.node, child_being_visited:0}];
        let next_node = node.node.children.first();
        let base_node = node;
        let mut this = Self {stack,next_node,base_node,fragment};
        this.descend_to_leaf();
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

    fn descend_to_leaf(&mut self) {
        if let Some(mut current) = std::mem::take(&mut self.next_node) {
            while self.can_descend(&current) && !current.node.children.is_empty() {
                self.stack.push(StackFrame { node: &current.node, child_being_visited: 0 });
                current = &current.node.children.first().unwrap();
            }
            self.next_node = Some(current);
        }
    }

    fn can_descend(&self, current_node:&node::Child) -> bool {
        match &self.fragment {
            TreeFragment::AllNodes               => true,
            TreeFragment::ChainAndDirectChildren => current_node.chained_with_parent,
        }
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use crate::builder::Builder;
    use crate::builder::TreeBuilder;



    #[test]
    fn leaf_iterating() {
        use ast::crumbs::InfixCrumb::*;
        use ast::crumbs::PrefixCrumb::*;
        use node::Kind::*;

        // Tree we use for tests (F means node which can be flattened):
        // root:                (-)
        //                    / |  \
        // children:        (F) ()  (F)
        //                 /|\      / | \
        // g-children:   ()()()  () () (F)
        //                   /|       / | \
        // gg-children:     ()()     ()() ()

        let tree = TreeBuilder::new(14)
            .add_child(0,10,Target,vec![LeftOperand])
                .chain_with_parent()
                .add_leaf (0,3,Target   ,vec![LeftOperand])
                .add_leaf (4,1,Operation,vec![Operator])
                .add_child(6,3,Argument ,vec![RightOperand])
                    .add_leaf(0,1,Operation,vec![Func])
                    .add_leaf(2,1,Target   ,vec![Arg])
                    .done()
                .done()
            .add_leaf (11,1,Operation,vec![Operator])
            .add_child(13,1,Target,vec![RightOperand])
                .chain_with_parent()
                .add_leaf (0,3,Target   ,vec![LeftOperand])
                .add_leaf (4,1,Operation,vec![Operator])
                .add_child(6,5,Argument ,vec![RightOperand])
                    .chain_with_parent()
                    .add_leaf(0,1,Target   ,vec![LeftOperand])
                    .add_leaf(2,1,Operation,vec![Operator])
                    .add_leaf(4,1,Argument ,vec![RightOperand])
                    .done()
                .done()
            .build();

        let root = tree.root_ref();

        // Whole tree iterating:
        let expected_crumbs = vec!
        [ vec![0,0]
        , vec![0,1]
        , vec![0,2,0]
        , vec![0,2,1]
        , vec![1]
        , vec![2,0]
        , vec![2,1]
        , vec![2,2,0]
        , vec![2,2,1]
        , vec![2,2,2]
        ];
        assert_eq!(expected_crumbs, root.clone().leaf_iter().map(|n| n.crumbs).collect_vec());

        // Chained children iterating:
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
