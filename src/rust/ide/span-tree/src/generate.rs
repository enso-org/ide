use crate::prelude::*;

use crate::Node;
use crate::NodeType;

use ast::{Ast, HasLength};
use data::text::Size;
use ast::assoc::Assoc;


#[derive(Clone,Debug,Eq,PartialEq)]
enum ChainingContext<'a> {
    None, Prefix, Operator(&'a str),
}

pub trait SpanTreeGeneratorTemplate : Sized {
    fn can_be_flatten   (&self, _chaining_ctx:&ChainingContext) -> bool               { false  }

    fn generate_children
    (&self, _gen:&mut ChildGenerator<Self>, _ctx:&ChainingContext) -> FallibleResult<()> {
        Ok(())
    }
}


pub trait SpanTreeGenerator {
    fn generate_node
    (&self, node_type:NodeType, offset:Size, chaining_ctx:&ChainingContext) -> Node;
}

impl<T> SpanTreeGenerator for T
where T : SpanTreeGeneratorTemplate + ast::crumbs::Crumbable + HasLength + CloneRef {
    fn generate_node
    (&self, node_type:NodeType, offset:Size, chaining_ctx:&ChainingContext) -> Node {
        let len                 = Size::new(self.len());
        let can_be_flatten      = self.can_be_flatten(chaining_ctx);
        let mut child_generator = ChildGenerator::new(self.clone_ref());
        self.generate_children(&mut child_generator,&chaining_ctx);
        let children = child_generator.children;
        Node {offset,len,node_type,children, chained_with_parent: can_be_flatten }
    }
}

#[derive(Debug)]
struct ChildGenerator<A> {
    ast            : A,
    current_offset : Size,
    children       : Vec<Node>,
}

impl<A> ChildGenerator<A> {
    fn new(ast:A) -> Self {
        let current_offset = default();
        let children       = default();
        Self {ast,current_offset,children}
    }

    fn spacing(&mut self, size:usize) {
        self.current_offset += Size::new(size);
    }
}

impl<A:ast::crumbs::TraversableAst> ChildGenerator<A> {
    fn generate
    (&mut self, crumbs:ast::Crumbs, chaining_ctx:&ChainingContext) -> FallibleResult<&Node> {
        let child_ast  = self.ast.get_traversing(&crumbs)?;
        let child_type = NodeType::AstChild{crumbs_from_parent:crumbs};
        let child      = child_ast.generate_node(child_type,self.current_offset,chaining_ctx);
        self.current_offset += child.len;
        self.children.push(child);
        Ok(self.children.last().unwrap())
    }

    fn generate_empty(&mut self) -> &Node {
        self.children.push(Node::new_empty(self.current_offset));
        self.children.last().unwrap()
    }
}

impl SpanTreeGenerator for Ast {
    fn generate_node
    (&self, _node_type:NodeType, _offset:Size, _chaining_ctx:&ChainingContext) -> Node {
        unimplemented!()
    }
}



// =============================================
// === Operator's AST (Infixes and Sections) ===
// =============================================

impl SpanTreeGeneratorTemplate for ast::known::Infix {
    fn can_be_flatten(&self, chaining_ctx:&ChainingContext) -> bool {
        let opr      = ast::known::Opr::try_from(&self.opr).ok();
        let opr_name = opr.as_ref().map(|opr| opr.name.as_str());
        match chaining_ctx {
            ChainingContext::Operator(opr) if opr_name == Some(opr) => true,
            _                                                       => false,
        }
    }

    fn generate_children
    (&self, gen:&mut ChildGenerator<Self>, ctx:&ChainingContext) -> FallibleResult<()> {
        let should_have_empty = !self.can_be_flatten(ctx);
        let opr_name          = ast::known::Opr::try_from(&self.opr)?.name;
        let assoc             = Assoc::of(opr_name.as_str());

        let left_ctx = match assoc {
            Assoc::Left  => ChainingContext::Operator(opr_name.as_str()),
            Assoc::Right => ChainingContext::None,
        };
        let right_ctx = match assoc {
            Assoc::Left  => ChainingContext::None,
            Assoc::Right => ChainingContext::Operator(opr_name.as_str()),
        };

        if should_have_empty && assoc == ast::assoc::Assoc::Right {
            gen.generate_empty();
        }
        use ast::crumbs::InfixCrumb::*;
        gen.generate(vec![LeftOperand.into()],&left_ctx)?;
        gen.spacing(self.loff);
        gen.generate(vec![Operator.into()],&ChainingContext::None)?;
        gen.spacing(self.roff);
        gen.generate(vec![RightOperand.into()],&right_ctx)?;
        if should_have_empty && assoc == ast::assoc::Assoc::Left {
            gen.generate_empty();
        }
        Ok(())
    }
}

impl SpanTreeGeneratorTemplate for ast::known::SectionLeft {
    fn can_be_flatten(&self, chaining_ctx:&ChainingContext) -> bool {
        let opr      = ast::known::Opr::try_from(&self.opr).ok();
        let opr_name = opr.map(|opr| opr.name.as_str());
        match chaining_ctx {
            ChainingContext::Operator(opr) if opr_name == Some(opr) => true,
            _                                                       => false,
        }
    }

    fn generate_children
    (&self, gen:&mut ChildGenerator<Self>, _:&ChainingContext) -> FallibleResult<()> {
        let opr_name = ast::known::Opr::try_from(&self.opr)?.name.as_str();
        let assoc    = Assoc::of(opr_name);
        let arg_ctx  = match assoc {
            Assoc::Left  => ChainingContext::Operator(opr_name),
            Assoc::Right => ChainingContext::None
        };
        use ast::crumbs::SectionLeftCrumb::*;
        gen.generate(vec![Arg.into()],&arg_ctx)?;
        gen.spacing(self.off);
        gen.generate(vec![Opr.into()],&ChainingContext::None)?;
        gen.generate_empty();
        Ok(())
    }
}

impl SpanTreeGeneratorTemplate for ast::known::SectionRight {
    fn can_be_flatten(&self, chaining_ctx:&ChainingContext) -> bool {
        let opr      = ast::known::Opr::try_from(&self.opr).ok();
        let opr_name = opr.map(|opr| opr.name.as_str());
        match chaining_ctx {
            ChainingContext::Operator(opr) if opr_name == Some(opr) => true,
            _                                                       => false,
        }
    }

    fn generate_children
    (&self, gen:&mut ChildGenerator<Self>, _:&ChainingContext) -> FallibleResult<()> {
        let opr_name = ast::known::Opr::try_from(&self.opr)?.name.as_str();
        let assoc    = Assoc::of(opr_name);
        let arg_ctx  = match assoc {
            Assoc::Right => ChainingContext::Operator(opr_name),
            Assoc::Left  => ChainingContext::None
        };
        use ast::crumbs::SectionLeftCrumb::*;
        gen.generate_empty();
        gen.generate(vec![Opr.into()],&ChainingContext::None)?;
        gen.spacing(self.off);
        gen.generate(vec![Arg.into()],&arg_ctx)?;
        Ok(())
    }
}

impl SpanTreeGeneratorTemplate for ast::known::SectionSides {
    fn generate_children
    (&self, gen:&mut ChildGenerator<Self>, _:&ChainingContext) -> FallibleResult<()> {
        gen.generate_empty();
        gen.generate(vec![ast::crumbs::SectionSidesCrumb.into()],&ChainingContext::None)?;
        gen.generate_empty();
        Ok(())
    }
}

// ===================
// === Application ===
// ===================

impl SpanTreeGeneratorTemplate for ast::known::Prefix {
    fn can_be_flatten(&self, chaining_ctx:&ChainingContext) -> bool {
        match chaining_ctx {
            ChainingContext::Prefix => true,
            _                       => false,
        }
    }

    fn generate_children
    (&self, gen:&mut ChildGenerator<Self>, ctx:&ChainingContext) -> FallibleResult<()> {
        let should_have_empty = !self.can_be_flatten(ctx);
        use ast::crumbs::PrefixCrumb::*;
        gen.generate(vec![Func.into()],&ChainingContext::Prefix)?;
        gen.spacing(self.off);
        gen.generate(vec![Arg.into()],&ChainingContext::None)?;
        if should_have_empty {
            gen.generate_empty();
        }
        Ok(())
    }
}

// ===========
// == Other ==
// ===========

