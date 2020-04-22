//! A module containing code related to SpanTree generation.

use crate::prelude::*;

use crate::node;
use crate::Node;
use crate::SpanTree;

use ast::Ast;
use ast::assoc::Assoc;
use ast::crumbs::Located;
use ast::HasLength;
use ast::opr::{GeneralizedInfix, Operand};
use data::text::Size;



// =============
// === Trait ===
// =============

/// A generation context, from which we can derive information of currently generated node kind and
/// if it will be chained with parent (see crate's doc for information about _chaining_).
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Context<'a> {
    /// Generated as a root node.
    Root,
    /// Generated as an argument of Infix or Prefix not being a target.
    Argument,
    /// Generated as a Function child of Prefix AST node.
    PrefixFunc,
    /// Generated as a first argument in PrefixChain.
    PrefixTarget,
    /// Generated as Operator child of Infix or Section AST node.
    Operator(&'a str),
    /// Generated as a Target child of Infix or Section AST node.
    OperatorTarget(&'a str),
}

/// A trait for all types from which we can generate referred SpanTree. Meant to be implemented for
/// all AST-like structures.
pub trait SpanTreeGenerator {
    /// Generate node with it's whole subtree.
    fn generate_node(&self, ctx:Context) -> FallibleResult<Node>;

    /// Generate tree for this AST treated as root for the whole expression.
    fn generate_tree(&self) -> FallibleResult<SpanTree> {
        Ok(SpanTree {
            root : self.generate_node(Context::Root)?
        })
    }
}


// =================
// === Utilities ===
// =================

// === Child Generator ===

/// An utility to generate children with increasing offsets.
#[derive(Debug,Default)]
struct ChildGenerator {
    current_offset : Size,
    children       : Vec<node::Child>,
}

impl ChildGenerator {
    /// Add spacing to current generator state. It will be taken into account for the next generated
    /// children's offsets
    fn spacing(&mut self, size:usize) {
        self.current_offset += Size::new(size);
    }

    fn generate_ast_node
    (&mut self, child_ast:Located<Ast>, ctx:Context)
    -> FallibleResult<&node::Child> {
        let child = node::Child {
            node                : child_ast.item.generate_node(ctx)?,
            offset              : self.current_offset,
            ast_crumbs          : child_ast.crumbs
        };
        self.current_offset += child.node.size;
        self.children.push(child);
        Ok(self.children.last().unwrap())
    }

    fn generate_empty_node(&mut self, kind:node::Kind) -> &node::Child {
        let child = node::Child {
            node                : Node::new_empty(kind),
            offset              : self.current_offset,
            ast_crumbs          : vec![]
        };
        self.children.push(child);
        self.children.last().unwrap()
    }
}



/// =============================
/// === Trait Implementations ===
/// =============================


// === AST ===

impl SpanTreeGenerator for Ast {
    fn generate_node(&self, ctx:Context) -> FallibleResult<Node> {
        use ast::known::*;

        if let Some(infix) = GeneralizedInfix::try_new_root(self) {
            infix.generate_node(ctx)
        } else {
            match self.shape() {
                ast::Shape::Prefix {..} =>
                    Prefix::try_new(self.clone_ref()).unwrap().generate_node(ctx),
                // TODO[a] add other shapes, e.g. macros
                _  => Ok(Node {
                    size     : Size::new(self.len()),
                    children : default(),
                    kind     : ast_node_kind(&ctx),
                }),
            }
        }
    }
}

fn ast_node_kind(chained:bool, ctx:&Context) -> node::Kind {
    match ctx {
        _ if chained               => node::Kind::Chained,
        Context::Root              => node::Kind::Root,
        Context::Argument          => node::Kind::Argument,
        Context::PrefixFunc        => node::Kind::Operation,
        Context::PrefixTarget      => node::Kind::Target,
        Context::Operator(_)       => node::Kind::Operation,
        Context::OperatorTarget(_) => node::Kind::Target,
    }
}

// === Operators (Sections and Infixes) ===

impl SpanTreeGenerator for GeneralizedInfix {

    fn generate_node(&self, ctx:Context) -> FallibleResult<Node> {
        let chained     = infix_can_be_chained_with_parent(self,ctx);
        let have_append = self.right.is_some() && !chained;
        let assoc       = self.assoc();
        let target_ctx  = Context::OperatorTarget(&self.opr.name);
        let located_opr = self.opr.clone().map(|opr| opr.ast().clone_ref());
        let opr_ctx     = Context::Operator(&self.opr.name);

        let (left_ctx,right_ctx) = match assoc {
            Assoc::Left  => (target_ctx       ,Context::Argument),
            Assoc::Right => (Context::Argument,target_ctx       ),
        };

        let mut gen = ChildGenerator::default();
        generate_arg_node(&self.left,Side::Left,&mut gen,left_ctx)?;
        gen.generate_ast_node(located_opr,opr_ctx)?;
        generate_arg_node(&self.right,Side::Right,&mut gen,right_ctx)?;
        if have_append { gen.generate_empty_node(node::Kind::Append); }

        Ok(Node {
            kind     : ast_node_kind(chained,&ctx),
            size     : gen.current_offset,
            children : gen.children,
        })
    }
}

enum Side {Left,Right}

fn generate_arg_node
(operand:&Operand, side:Side, gen:&mut ChildGenerator, ctx:Context) -> FallibleResult<()> {
    match (operand,side) {
        (Some(arg),Side::Left) => {
            gen.generate_ast_node(arg.wrapped.clone(),ctx)?;
            gen.spacing(arg.off);
        }
        (Some(arg),Side::Right) => {
            gen.spacing(arg.off);
            gen.generate_ast_node(arg.wrapped.clone(),ctx)?;
        }
        (None,_) => {
            gen.generate_empty_node(node::Kind::Missing)?;
        }
    }
    Ok(())
}

// === Application ===

impl SpanTreeGenerator for ast::known::Prefix {

    fn generate_node(&self, ctx:Context) -> FallibleResult<Node> {
        let chained     = prefix_can_be_chained_with_parent(ctx);
        let have_append = !chained;
        let func_ast    = Located::new(Func,self.func.clone_ref());

        use ast::crumbs::PrefixCrumb::*;
        let mut gen = ChildGenerator::default();
        let func    = gen.generate_ast_node(func_ast,Context::PrefixFunc)?;
        // The target is a fist argument in a prefix chain. So in our case the arg is a target only
        // if func is a chained node.
        let arg_ctx = match &func.node.kind {
            node::Kind::Chained => Context::Argument,
            _                   => Context::PrefixTarget,
        };
        gen.spacing(self.off);
        gen.generate_ast_node(Located::new(Arg,self.arg.clone_ref()),arg_ctx)?;
        if have_append {
            gen.generate_empty_node(node::Kind::Append);
        }

        Ok(Node {
            kind     : ast_node_kind(chained,&ctx),
            size     : Size::new(self.len()),
            children : gen.children,
        })
    }
}



// ===========================
// === Chaining Conditions ===
// ===========================

fn ast_can_be_chained_with_parent(ast:&Ast, ctx:Context) -> bool {
    if let Some(infix) = GeneralizedInfix::try_new(&Located::new_root(ast.clone_ref())) {
        infix_can_be_chained_with_parent(&infix,ctx)
    } else {
        match ast.shape() {
            ast::Shape::Prefix {..} => prefix_can_be_chained_with_parent(ctx),
            _                       => false,
        }
    }
}

fn infix_can_be_chained_with_parent(infix:&GeneralizedInfix, ctx:Context) -> bool {
    match ctx {
        Context::OperatorTarget(name) if infix.opr.item.name == *name => true,
        _                                                             => false,
    }
}

fn prefix_can_be_chained_with_parent(ctx:Context) -> bool {
    match ctx {
        Context::PrefixFunc => true,
        _                   => false,
    }
}



// ============
// === Test ===
// ============

#[cfg(test)]
mod test {
    use super::*;

    use crate::builder::TreeBuilder;
    use crate::node::Kind::*;

    use ast::crumbs::InfixCrumb;
    use ast::crumbs::PrefixCrumb;
    use ast::crumbs::SectionLeftCrumb;
    use ast::crumbs::SectionRightCrumb;
    use ast::crumbs::SectionSidesCrumb;
    use parser::Parser;

    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn generating_span_tree() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line("2 + foo bar - 3").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = TreeBuilder::new(15)
            .add_child(0,11,Target,vec![InfixCrumb::LeftOperand])
                .add_leaf (0,1,Target   ,vec![InfixCrumb::LeftOperand])
                .add_leaf (2,1,Operation,vec![InfixCrumb::Operator])
                .add_child(4,7,Argument ,vec![InfixCrumb::RightOperand])
                    .add_leaf(0,3,Operation,vec![PrefixCrumb::Func])
                    .add_leaf(4,3,Target   ,vec![PrefixCrumb::Arg])
                    .add_empty_child(7,Append)
                    .done()
                .add_empty_child(11,Append)
                .done()
            .add_leaf(12,1,Operation,vec![InfixCrumb::Operator])
            .add_leaf(14,1,Argument,vec![InfixCrumb::RightOperand])
            .add_empty_child(15,Append)
            .build();

        assert_eq!(expected,tree)
    }

    #[wasm_bindgen_test]
    fn generate_span_tree_with_chains() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line("2 + 3 + foo bar baz 13 + 5").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = TreeBuilder::new(26)
            .add_child(0,22,Chained,vec![InfixCrumb::LeftOperand])
                .add_child(0,5,Chained,vec![InfixCrumb::LeftOperand])
                    .add_leaf(0,1,Target,   vec![InfixCrumb::LeftOperand])
                    .add_leaf(2,1,Operation,vec![InfixCrumb::Operator])
                    .add_leaf(4,1,Argument ,vec![InfixCrumb::RightOperand])
                    .done()
                .add_leaf (6,1 ,Operation,vec![InfixCrumb::Operator])
                .add_child(8,14,Argument ,vec![InfixCrumb::RightOperand])
                    .add_child(0,11,Chained,vec![PrefixCrumb::Func])
                        .add_child(0,7,Chained,vec![PrefixCrumb::Func])
                            .add_leaf(0,3,Operation,vec![PrefixCrumb::Func])
                            .add_leaf(4,3,Target   ,vec![PrefixCrumb::Arg])
                            .done()
                        .add_leaf(8,3,Argument,vec![PrefixCrumb::Arg])
                        .done()
                    .add_leaf(12,2,Argument,vec![PrefixCrumb::Arg])
                    .add_empty_child(14,Append)
                    .done()
                .done()
            .add_leaf(23,1,Operation,vec![InfixCrumb::Operator])
            .add_leaf(25,1,Argument ,vec![InfixCrumb::RightOperand])
            .add_empty_child(26,Append)
            .build();

        assert_eq!(expected,tree);
    }

    #[wasm_bindgen_test]
    fn generating_span_tree_from_right_assoc_operator() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line("1,2,3").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = TreeBuilder::new(5)
            .add_leaf (0,1,Argument ,vec![InfixCrumb::LeftOperand])
            .add_leaf (1,1,Operation,vec![InfixCrumb::Operator])
            .add_child(2,3,Chained  ,vec![InfixCrumb::RightOperand])
                .add_leaf(0,1,Argument ,vec![InfixCrumb::LeftOperand])
                .add_leaf(1,1,Operation,vec![InfixCrumb::Operator])
                .add_leaf(2,1,Target   ,vec![InfixCrumb::RightOperand])
                .done()
            .add_empty_child(3,Append)
            .build();

        assert_eq!(expected,tree)
    }

    #[wasm_bindgen_test]
    fn generating_span_tree_from_section() {
        let parser = Parser::new_or_panic();
        // The star makes `SectionSides` ast being one of the parameters of + chain. First + makes
        // SectionRight, and last + makes SectionLeft.
        let ast    = parser.parse_line("+ * + 2 +").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = TreeBuilder::new(9)
            .add_child(0,7,Chained,vec![SectionLeftCrumb::Arg])
                .add_child(0,3,Chained,vec![InfixCrumb::LeftOperand])
                    .add_empty_child(0,Missing)
                    .add_leaf (0,1,Operation,vec![SectionRightCrumb::Opr])
                    .add_child(2,1,Argument ,vec![SectionRightCrumb::Arg])
                        .add_empty_child(0,Missing)
                        .add_leaf(0,1,Operation,vec![SectionSidesCrumb])
                        .add_empty_child(1,Missing)
                        .done()
                    .done()
                .add_leaf(4,1,Operation,vec![InfixCrumb::Operator])
                .add_leaf(6,1,Argument ,vec![InfixCrumb::RightOperand])
                .done()
            .add_leaf(8,1,Operation,vec![SectionLeftCrumb::Opr])
            .add_empty_child(9,Missing)
            .build();

        assert_eq!(expected,tree);
    }

    #[wasm_bindgen_test]
    fn generating_span_tree_from_right_assoc_section() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line(",2,").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = TreeBuilder::new(3)
            .add_empty_child(0,Missing)
            .add_leaf (0,1,Operation,vec![SectionRightCrumb::Opr])
            .add_child(1,2,Chained  ,vec![SectionRightCrumb::Arg])
                .add_leaf(0,1,Argument ,vec![SectionLeftCrumb::Arg])
                .add_leaf(1,1,Operation,vec![SectionLeftCrumb::Opr])
                .add_empty_child(2,Missing)
                .done()
            .build();

        assert_eq!(expected,tree);
    }
}
