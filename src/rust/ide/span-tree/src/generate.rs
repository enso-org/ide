use crate::prelude::*;

use crate::node;
use crate::Node;
use crate::node::NodeType;

use ast::Ast;
use data::text::Index;
use data::text::Size;
use data::text::Span;


pub trait SpanTreeGenerator : ast::crumbs::Crumbable {
    fn generate_node(&self, node_type:node::NodeType, offset:Size) -> Node;
}

#[derive(Clone,Debug,Eq,PartialEq)]
enum ChainingContext {
    None, Prefix, Operator(String),
}

#[derive(Debug)]
struct ChildGenerator<A> {
    ast            : A,
    current_offset : Size,
}

impl<A> ChildGenerator<A> {
    fn new(ast:A) -> Self {
        let current_offset = default();
        Self {ast,current_offset}
    }

    fn spacing(&mut self, size:usize) {
        self.current_offset += Size::new(size);
    }

    fn finish(self) -> Size {
        self.current_offset
    }
}

impl<A:ast::crumbs::TraversableAst> ChildGenerator<A> {
    fn next(&mut self, crumbs:ast::Crumbs) -> FallibleResult<Node> {
        let child_ast  = self.ast.get_traversing(&crumbs)?;
        let child_type = node::NodeType::AstChild(crumbs);
        let child      = child_ast.generate_node(child_type,self.current_offset);
        self.current_offset += child.len;
        Ok(child)
    }

    fn next_empty(&self) -> Node {
        Node::new_empty(self.current_offset)
    }
}

impl SpanTreeGenerator for Ast {
    fn generate_node(&self, node_type: NodeType, offset: Size) -> Node {
        unimplemented!()
    }
}

impl SpanTreeGenerator for ast::known::Infix {
    fn generate_node(&self, node_type: NodeType, offset: Size) -> Node {
        use ast::crumbs::InfixCrumb::*;

        let mut child_gen = ChildGenerator::new(self.clone_ref());
        let larg          = child_gen.next(vec![LeftOperand.into()]).unwrap();
        child_gen.spacing(self.loff);
        let opr = child_gen.next(vec![Operator.into()]).unwrap();
        child_gen.spacing(self.roff);
        let rarg     = child_gen.next(vec![RightOperand.into()]).unwrap();
        let empty    = child_gen.next_empty();
        let len      = child_gen.finish();

        let children = vec![larg,opr,rarg,empty];
        Node { node_type,offset,len,children }
    }
}

impl SpanTreeGenerator for ast::known::SectionLeft {
    fn generate_node(&self, node_type: NodeType, offset: Size) -> Node {
        use ast::crumbs::SectionLeftCrumb::*;

        let mut child_gen = ChildGenerator::new(self.clone_ref());
        let arg           = child_gen.next(vec![Arg.into()]).unwrap();
        child_gen.spacing(self.off);
        let opr           = child_gen.next(vec![Opr.into()]).unwrap();
        let empty    = child_gen.next_empty();
        let len      = child_gen.finish();

        let children = vec![arg,opr,empty];
        Node { node_type,offset,len,children }
    }
}

impl SpanTreeGenerator for ast::known::SectionRight {
    fn generate_node(&self, node_type: NodeType, offset: Size) -> Node {
        use ast::crumbs::SectionRightCrumb::*;

        let mut child_gen = ChildGenerator::new(self.clone_ref());
        let empty         = child_gen.next_empty();
        let opr           = child_gen.next(vec![Opr.into()]).unwrap();
        child_gen.spacing(self.off);
        let arg           = child_gen.next(vec![Arg.into()]).unwrap();
        let len           = child_gen.finish();

        let children = vec![empty,opr,arg];
        Node { node_type,offset,len,children }
    }
}

impl SpanTreeGenerator for ast::known::SectionSides {
    fn generate_node(&self, node_type: NodeType, offset: Size) -> Node {
        let mut child_gen = ChildGenerator::new(self.clone_ref());
        let lempty         = child_gen.next_empty();
        let opr           = child_gen.next(vec![ast::crumbs::SectionSidesCrumb.into()]).unwrap();
        let rempty         = child_gen.next_empty();
        let len           = child_gen.finish();

        let children = vec![lempty,opr,rempty];
        Node { node_type,offset,len,children }
    }
}