//! Module providing advanced iterators over SpanTree nodes.

use crate::Node;
use crate::NodeRef;
use crate::tree;



// ===============================
// === Chain Children Iterator ===
// ===============================

struct ChainStack<'a> {
    node                : &'a Node,
    child_being_visited : usize,
}

pub struct ChainChildrenIterator<'a> {
    stack     : Vec<ChainStack<'a>>,
    next_node : Option<&'a tree::Child>,
    base_node : NodeRef<'a>,
}

impl<'a> Iterator for ChainChildrenIterator<'a> {
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_node.is_some() {
            let crumbs       = self.stack.iter().map(|sf| sf.child_being_visited);
            let return_value = self.base_node.clone().traverse_subnode(crumbs);
            self.make_dfs_step();
            self.search_for_not_chained();
            return_value
        } else {
            None
        }
    }
}

impl<'a> ChainChildrenIterator<'a> {
    pub fn new(node: NodeRef<'a>) -> Self {
        let stack     = vec![ChainStack{node:&node.node, child_being_visited:0}];
        let next_node = node.node.children.first();
        let base_node = node;
        let mut this = Self {stack,next_node,base_node};
        this.search_for_not_chained();
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

    fn search_for_not_chained(&mut self) {
        if let Some(mut current) = std::mem::take(&mut self.next_node) {
            while current.chained_with_parent && !current.node.children.is_empty() {
                self.stack.push(ChainStack { node: &current.node, child_being_visited: 0 });
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
    use crate::builder::RootBuilder;
    use crate::SpanTree;


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

        let root = RootBuilder::new(14)
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

        let tree = SpanTree{root};
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
