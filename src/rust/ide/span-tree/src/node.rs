use crate::prelude::*;

use data::text::Size;
use data::text::Span;
use crate::node::DfsMode::OneLevelFlatten;


// =============
// === Nodes ===
// =============

#[derive(Debug)]
pub enum NodeType {
    Root, AstChild(ast::Crumbs), EmptyChild
}

// === Node ===

#[derive(Debug)]
pub struct Node {
    pub offset         : Size,
    pub len            : Size,
    pub node_type      : NodeType,
    pub children       : Vec<Node>,
    pub can_be_flatten : bool,
}

impl Node {
    pub fn new_empty(offset:Size) -> Self {
        let node_type      = NodeType::EmptyChild;
        let len            = Size::new(0);
        let children       = Vec::new();
        let can_be_flatten = false;
        Node {node_type, offset,len,children,can_be_flatten}
    }
}


// === Node Reference ===

#[derive(Clone,Debug)]
pub struct NodeRef<'a> {
    pub node              : &'a Node,
    pub crumbs            : Vec<usize>,
    pub parent_ast_crumbs : ast::Crumbs,
}

impl<'a> NodeRef<'a> {
    pub fn reborrow(&self) -> Self {
        self.clone()
    }

    pub fn ast_crumbs(&self) -> Option<ast::Crumbs> {
        match &self.node.node_type {
            NodeType::Root                  => Some(default()),
            NodeType::EmptyChild            => None,
            NodeType::AstChild(from_parent) => {
                let mut crumbs = self.parent_ast_crumbs.clone();
                crumbs.extend(from_parent.iter());
                Some(crumbs)
            },
        }
    }

    pub fn child(mut self, index:usize) -> Option<NodeRef<'a>> {
        self.node.children.get(index).map(|child| {
            if let NodeType::AstChild(ast) = &self.node.node_type {
                self.parent_ast_crumbs.extend(ast);
            }
            self.crumbs.push(index);
            self.node = child;
            self
        })
    }

    pub fn traverse_subnode(self, crumbs:impl IntoIterator<Item=usize>) -> Option<NodeRef<'a>> {
        let mut iter = crumbs.into_iter();
        match iter.next() {
            Some(index) => self.child(index).and_then(|child| child.traverse_subnode(iter)),
            None        => Some(self)
        }
    }
}


// === Root Node ===

#[derive(Debug,Shrinkwrap)]
pub struct RootNode(Node);

// pub type Crumbs = Vec<usize>;

impl RootNode {
    pub fn dfs_iter(&self) -> DfsIterator {
        DfsIterator::new(self.as_ref(),DfsMode::All)
    }

    pub fn dfs_iter_flatten(&self) -> DfsIterator {
        DfsIterator::new(self.as_ref(),DfsMode::AllFlatten)
    }

    pub fn children_iter_flatten(&self) -> DfsIterator {
        DfsIterator::new(self.as_ref(),DfsMode::OneLevelFlatten)
    }

    pub fn get_node(&self, crumbs:impl IntoIterator<Item=usize>) -> Option<NodeRef> {
        self.as_ref().traverse_subnode(crumbs)
    }

    fn as_ref(&self) -> NodeRef {
        let RootNode(node)    = self;
        let crumbs            = default();
        let parent_ast_crumbs = default();
        NodeRef {node,crumbs,parent_ast_crumbs}
    }
}



// ====================
// === DFS Iterator ===
// ====================

struct DfsStackItem<'a> {
    node           : &'a Node,
    visiting_child : usize,
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
enum DfsMode {All,AllFlatten,OneLevelFlatten}

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
            let return_value = self.root.reborrow().traverse_subnode(crumbs);
            self.make_dfs_step();
            while self.should_skip_node() { self.make_dfs_step() }
            return_value
        } else {
            None
        }
    }
}

impl<'a> DfsIterator<'a> {
    fn new(root:NodeRef<'a>, mode:DfsMode) -> Self {
        let stack = default();
        let next_node = Some(root.node);
        Self {stack,next_node,root,mode}
    }

    fn make_dfs_step(&mut self) {
        if let Some(current) = std::mem::take(&mut self.next_node) {
            let descension_allowed = self.mode != OneLevelFlatten || current.can_be_flatten;
            if !current.children.is_empty() && descension_allowed {
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
        let should_flatten = |n:&Node| n.can_be_flatten && n.children.is_empty();
        flatten_mode && self.next_node.map_or(false, should_flatten)
    }
}


// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dfs_iterator() {
        let grand_child1 = ast_child_node(vec![ast::crumbs::PrefixCrumb::Func.into()], 0, 3);
        let grand_child2 = ast_child_node(vec![ast::crumbs::PrefixCrumb::Arg.into()] , 4, 3);
        let grand_child3 = Node::new_empty(Size::new(7));
        let grand_child4 = ast_child_node(vec![ast::crumbs::PrefixCrumb::Func.into()], 0, 3);
        let grand_child5 = ast_child_node(vec![ast::crumbs::PrefixCrumb::Arg.into()] , 4, 3);
        let mut child1   = ast_child_node(vec![ast::crumbs::InfixCrumb::LeftOperand.into()] , 0, 1);
        let child2       = ast_child_node(vec![ast::crumbs::InfixCrumb::Operator.into()]    , 2, 1);
        let mut child3   = ast_child_node(vec![ast::crumbs::InfixCrumb::RightOperand.into()], 4, 7);

        child1.children = vec![grand_child1,grand_child2,grand_child3];
        child3.children = vec![grand_child4,grand_child5];
        let root = RootNode(Node {
            offset         : Size::new(0),
            len            : Size::new(11),
            node_type      : NodeType::Root,
            children       : vec![child1, child2, child3],
            can_be_flatten : false,
        });

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

        use ast::crumbs::InfixCrumb::*;
        use ast::crumbs::PrefixCrumb::*;
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
        assert_eq!(expected_ast_crumbs, root.dfs_iter().map(|rch| rch.ast_crumbs()).collect_vec())
    }

    fn ast_child_node(crumbs:ast::Crumbs, offset:usize, len:usize) -> Node {
        Node {
            node_type      : NodeType::AstChild(crumbs),
            offset         : Size::new(offset),
            len            : Size::new(len),
            children       : default(),
            can_be_flatten : false,
        }
    }
}