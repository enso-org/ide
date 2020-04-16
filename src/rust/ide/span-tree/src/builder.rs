//! An utility builder to be used in tests.

use crate::prelude::*;

use crate::{Node, SpanTree};
use crate::Type;
use data::text::Size;
use crate::tree::Child;

pub trait Builder : Sized {
    fn built_node(&mut self) -> &mut Node;

    fn add_ast_child<Cbs>(self, offset:usize, len:usize, crumbs:Cbs) -> ChildBuilder<Self>
    where Cbs : IntoIterator<Item:Into<ast::crumbs::Crumb>> {
        let node = Node {
            node_type: Type::Ast,
            len: Size::new(len),
            children: vec![]
        };
        let child = Child { node,
            offset              : Size::new(offset),
            chained_with_parent : false,
            ast_crumbs          : crumbs.into_iter().map(|cb| cb.into()).collect(),
        };
        ChildBuilder {
            built  : child,
            parent : self
        }
    }

    fn add_ast_leaf<Cbs>(self, offset:usize, len:usize, crumbs:Cbs) -> Self
    where Cbs : IntoIterator<Item:Into<ast::crumbs::Crumb>> {
        self.add_ast_child(offset,len,crumbs).done()
    }

    fn add_empty_child(mut self, offset:usize) -> Self {
        let node = Node::new_empty();
        let child = Child { node,
            offset : Size::new(offset),
            chained_with_parent : false,
            ast_crumbs          : vec![]
        };
        self.built_node().children.push(child);
        self
    }
}

pub struct RootBuilder {
    built : Node,
}

pub struct ChildBuilder<Parent> {
    built  : Child,
    parent : Parent,
}


impl<Parent:Builder> ChildBuilder<Parent> {

    pub fn chain_with_parent(mut self) -> Self {
        self.built.chained_with_parent = true;
        self
    }

    pub fn done(mut self) -> Parent {
        self.parent.built_node().children.push(self.built);
        self.parent
    }
}

impl<T> Builder for ChildBuilder<T> {
    fn built_node(&mut self) -> &mut Node {
        &mut self.built.node
    }
}

impl RootBuilder {
    pub fn new(len:usize) -> Self {
        RootBuilder {
            built : Node {
                node_type: Type::Ast,
                len: Size::new(len),
                children: vec![],
            }
        }
    }

    pub fn build(self) -> SpanTree {
        SpanTree {
            root : self.built
        }
    }
}

impl Builder for RootBuilder {
    fn built_node(&mut self) -> &mut Node {
        &mut self.built
    }
}