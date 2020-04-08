use crate::prelude::*;

use data::text::Size;
use data::text::Span;



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
    pub offset    : Size,
    pub len       : Size,
    pub node_type : NodeType,
    pub children  : Vec<Node>,
}

impl Node {
    pub fn new_empty(offset:Size) -> Self {
        let node_type  = NodeType::EmptyChild;
        let len        = Size::new(0);
        let children   = Vec::new();
        Node {node_type, offset,len,children}
    }
}


// === Node Reference ===

pub struct NodeRef<'a> {
    pub node              : &'a Node,
    pub crumbs            : Vec<usize>,
    pub parent_ast_crumbs : ast::Crumbs,
}

impl<'a> NodeRef<'a> {
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
}


// === Root Node ===

#[derive(Debug,Shrinkwrap)]
pub struct RootNode(Node);

// pub type Crumbs = Vec<usize>;

impl RootNode {
    pub fn dfs_iter(&self) -> DfsIterator {
        DfsIterator {
            next_crumb: Some(vec![]),
            root: &self
        }
    }

    pub fn get_node(&self, crumbs:Vec<usize>) -> Option<NodeRef> {
        let RootNode(node) = self;
        let level          = 0;
        let initial_ast    = Vec::new();
        Self::get_subnode(node,crumbs,level,initial_ast)
    }

    pub fn get_subnode
    (node:&Node, crumbs:Vec<usize>, level:usize, mut ast_so_far:ast::Crumbs) -> Option<NodeRef> {
        let remaining = &crumbs[level..];
        if remaining.is_empty() {
            let parent_ast_crumbs = ast_so_far;
            Some(NodeRef {node,crumbs,parent_ast_crumbs})
        } else {
            let child = node.children.get(remaining[0])?;
            let level = level + 1;
            if let NodeType::AstChild(ast) = &node.node_type {
                ast_so_far.extend(ast);
            }
            Self::get_subnode(child, crumbs, level, ast_so_far)
        }
    }
}



// ====================
// === DFS Iterator ===
// ====================

pub struct DfsIterator<'a> {
    next_crumb : Option<Vec<usize>>,
    root       : &'a RootNode,
}

impl<'a> Iterator for DfsIterator<'a> {
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        std::mem::take(&mut self.next_crumb).map(|mut crumbs| {
            let current      = crumbs.clone();
            let current_node = self.root.get_node(current).unwrap();
            if !current_node.node.children.is_empty() {
                crumbs.push(0);
                self.next_crumb = Some(crumbs);
            } else {
                while !crumbs.is_empty() {
                    let last         = crumbs.pop().unwrap();
                    let parent       = self.root.get_node(std::mem::take(&mut crumbs)).unwrap();
                    let children_len = parent.node.children.len();
                    crumbs           = parent.crumbs;
                    let sibling      = last + 1;
                    if children_len > sibling {
                        crumbs.push(sibling);
                        self.next_crumb = Some(crumbs);
                        break;
                    }
                }
            }
            current_node
        })
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
            offset   : Size::new(0),
            len      : Size::new(11),
            node_type: NodeType::Root,
            children : vec![child1, child2, child3],
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
            node_type : NodeType::AstChild(crumbs),
            offset    : Size::new(offset),
            len       : Size::new(len),
            children  : default()
        }
    }
}