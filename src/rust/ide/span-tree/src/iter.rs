use crate::prelude::*;

use crate::Node;
use crate::NodeRef;



// ====================
// === DFS Iterator ===
// ====================

struct DfsStackItem<'a> {
    node           : &'a Node,
    visiting_child : usize,
}

/// A mode of this DfsSearching.
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum DfsMode {
    /// Go through whole tree as it is.
    All,
    /// Go through whole tree but skip nodes with `can_be_flatten` flag set.
    AllFlatten,
    /// Iterate over children of root node taking into account
    OneLevelFlatten
}

impl DfsMode {
    fn is_flatten(&self) -> bool {
        match self {
            Self::AllFlatten | Self::OneLevelFlatten => true,
            Self::All                                => false,
        }
    }
}

pub struct DfsIterator<'a> {
    stack     : Vec<DfsStackItem<'a>>,
    next_node : Option<&'a Node>,
    root      : NodeRef<'a>,
    mode      : DfsMode,
}

impl<'a> Iterator for DfsIterator<'a> {
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_node.is_some() {
            let crumbs       = self.stack.iter().map(|sf| sf.visiting_child);
            let return_value = self.root.clone().traverse_subnode(crumbs);
            self.make_dfs_step();
            while self.should_skip_node() { self.make_dfs_step() }
            return_value
        } else {
            None
        }
    }
}

impl<'a> DfsIterator<'a> {
    pub fn new(root:NodeRef<'a>, mode:DfsMode) -> Self {
        let stack = default();
        let next_node = Some(root.node);
        Self {stack,next_node,root,mode}
    }

    fn make_dfs_step(&mut self) {
        if let Some(current) = std::mem::take(&mut self.next_node) {
            if !current.children.is_empty() && self.can_descend(current) {
                self.next_node = Some(current.children.first().unwrap());
                self.stack.push(DfsStackItem{node:current, visiting_child:0});
            } else {
                while self.next_node.is_none() && !self.stack.is_empty() {
                    let parent = self.stack.last_mut().unwrap();
                    parent.visiting_child += 1;
                    self.next_node = parent.node.children.get(parent.visiting_child);
                    if self.next_node.is_none() {
                        self.stack.pop();
                    }
                }
            }
        }
    }

    fn should_skip_node(&self) -> bool {
        let flatten_mode   = self.mode.is_flatten();
        let should_flatten = |n:&Node| n.chained_with_parent && !n.children.is_empty();
        flatten_mode && self.next_node.map_or(false, should_flatten)
    }

    fn can_descend(&self, current:&Node) -> bool {
        self.mode != DfsMode::OneLevelFlatten || self.stack.is_empty() || current.chained_with_parent
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Node;
    use crate::NodeType;
    use crate::SpanTree;

    use data::text::Size;

    #[test]
    fn dfs_iterator() {
        use ast::crumbs::InfixCrumb::*;
        use ast::crumbs::PrefixCrumb::*;
        // Tree we use for tests (E - Empty child, R - Root):
        // root:                (R)
        //                    / |  \
        // children:        ()  ()  ()
        //                 /|\     / \
        // g-children:   ()()(E)  () ()

        let grand_child1 = ast_child_node(vec![Func.into()], 0, 3);
        let grand_child2 = ast_child_node(vec![Arg.into()] , 4, 3);
        let grand_child3 = Node::new_empty(Size::new(7));
        let grand_child4 = ast_child_node(vec![Func.into()], 0, 3);
        let grand_child5 = ast_child_node(vec![Arg.into()] , 4, 3);
        let mut child1   = ast_child_node(vec![LeftOperand.into()] , 0, 7);
        let child2       = ast_child_node(vec![Operator.into()]    , 8, 1);
        let mut child3   = ast_child_node(vec![RightOperand.into()], 10,7);

        child1.children = vec![grand_child1,grand_child2,grand_child3];
        child3.children = vec![grand_child4,grand_child5];
        let root = Node {
            offset         : Size::new(0),
            len            : Size::new(11),
            node_type      : NodeType::Root,
            children       : vec![child1, child2, child3],
            chained_with_parent: false,
        };
        let tree = SpanTree{root};
        let root = tree.root_ref();

        let expected_crumbs = vec!
        [ vec![]
          , vec![0]
          , vec![0,0]
          , vec![0,1]
          , vec![0,2]
          , vec![1]
          , vec![2]
          , vec![2,0]
          , vec![2,1]
        ];
        assert_eq!(expected_crumbs, root.dfs_iter().map(|rch| rch.crumbs).collect_vec());

        let expected_ast_crumbs = vec!
        [ Some(vec![])
          , Some(vec![LeftOperand.into()])
          , Some(vec![LeftOperand.into(), Func.into()])
          , Some(vec![LeftOperand.into(), Arg.into()])
          , None
          , Some(vec![Operator.into()])
          , Some(vec![RightOperand.into()])
          , Some(vec![RightOperand.into(), Func.into()])
          , Some(vec![RightOperand.into(), Arg.into()])
        ];
        assert_eq!(expected_ast_crumbs, root.dfs_iter().map(|rch| rch.ast_crumbs()).collect_vec());

        let expected_indices = vec![0,0,0,4,7,8,10,10,14];
        assert_eq!(expected_indices, root.dfs_iter().map(|rch| rch.span_index.value).collect_vec());
    }

    #[test]
    fn flatten_iterating() {
        use ast::crumbs::InfixCrumb::*;
        use ast::crumbs::PrefixCrumb::*;

        // Tree we use for tests (F means node which can be flattened):
        // root:                (-)
        //                    / |  \
        // children:        (F) ()  (F)
        //                 /|      / | \
        // g-children:   ()()    () () (F)
        //                            / | \
        // gg-children:              ()() ()

        // Level 4. (Grand-grand children)
        let gg_child1    = ast_child_node(vec![Func.into()],0,1);
        let gg_child2    = ast_child_node(vec![Arg.into()] ,2,1);
        let gg_child3    = ast_child_node(vec![LeftOperand.into()] ,0,1);
        let gg_child4    = ast_child_node(vec![Operator.into()]    ,2,1);
        let gg_child5    = ast_child_node(vec![RightOperand.into()],4,1);
        // Level 3. (Grand children)
        let g_child1     = ast_child_node(vec![LeftOperand.into()] ,0,3);
        let g_child2     = ast_child_node(vec![Operator.into()]    ,4,1);
        let mut g_child3 = ast_child_node(vec![RightOperand.into()],6,3);
        let g_child4     = ast_child_node(vec![LeftOperand.into()] ,0,3);
        let g_child5     = ast_child_node(vec![Operator.into()]    ,4,1);
        let mut g_child6 = ast_child_node(vec![RightOperand.into()],6,3);
        // Level 2. (children)
        let mut child1   = ast_child_node(vec![LeftOperand.into()] ,0 ,10);
        let child2       = ast_child_node(vec![Operator.into()]    ,11,1 );
        let mut child3   = ast_child_node(vec![RightOperand.into()],13,1);
        g_child3.children       = vec![gg_child1,gg_child2];
        g_child6.children       = vec![gg_child3,gg_child4,gg_child5];
        g_child6.chained_with_parent = true;
        child1.children         = vec![g_child1,g_child2,g_child3];
        child1.chained_with_parent = true;
        child3.children         = vec![g_child4,g_child5,g_child6];
        child3.chained_with_parent = true;
        // Level 1. (root)
        let root = Node {
            offset         : Size::new(0),
            len            : Size::new(11),
            node_type      : NodeType::Root,
            children       : vec![child1, child2, child3],
            chained_with_parent: false,
        };

        let tree = SpanTree{root};
        let root = tree.root_ref();

        // Dfs chains iterating
        let expected_crumbs = vec!
            [ vec![]
            , vec![0,0]
            , vec![0,1]
            , vec![0,2]
            , vec![0,2,0]
            , vec![0,2,1]
            , vec![1]
            , vec![2,0]
            , vec![2,1]
            , vec![2,2]
            ];
        assert_eq!(expected_crumbs, root.chains_dfs_iter().map(|n| n.crumbs).collect_vec());

        // Chain children iterating
        let expected_crumbs = vec!
            [ vec![0,0]
            , vec![0,1]
            , vec![0,2]
            , vec![1]
            , vec![2,0]
            , vec![2,1]
            , vec![2,2]
            ];
        assert_eq!(expected_crumbs, root.chains_dfs_iter().map(|n| n.crumbs).collect_vec());
    }

    fn ast_child_node(crumbs_from_parent:ast::Crumbs, offset:usize, len:usize) -> Node {
        Node {
            node_type      : NodeType::AstChild{crumbs_from_parent},
            offset         : Size::new(offset),
            len            : Size::new(len),
            children       : default(),
            chained_with_parent: false,
        }
    }
}
