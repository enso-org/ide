//! A module containing code related to SpanTree generation.

use crate::prelude::*;

use crate::node;
use crate::Node;
use crate::SpanTree;

use ast::Ast;
use ast::assoc::Assoc;
use ast::crumbs::{Located, PrefixCrumb};
use ast::HasLength;
use ast::opr::{GeneralizedInfix, Operand};
use data::text::Size;
use crate::node::Kind::Chained;
use ast::crumbs::InfixCrumb::LeftOperand;
use ast::Shape::Cons;


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
    OperatorTarget(&'a str)
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
    (&mut self, child_ast:Located<Ast>, ctx:Context) -> FallibleResult<&node::Child> {
        let node = child_ast.item.generate_node(ctx)?;
        Ok(self.add_node(child_ast.crumbs,node))
    }

    fn add_node(&mut self, ast_crumbs:ast::Crumbs, node:Node) -> &node::Child {
        let offset = self.current_offset;
        let child = node::Child {node,ast_crumbs,offset};
        self.current_offset += child.node.size;
        self.children.push(child);
        self.children.last().unwrap()
    }

    fn generate_empty_node(&mut self) -> &node::Child {
        let child = node::Child {
            node                : Node::new_empty(),
            offset              : self.current_offset,
            ast_crumbs          : vec![]
        };
        self.children.push(child);
        self.children.last().unwrap()
    }

    fn reverse_children(&mut self) {
        self.children.reverse();
        for child in &mut self.children {
            child.offset = self.current_offset - child.offset - child.node.size;
        }
    }
}



/// =============================
/// === Trait Implementations ===
/// =============================


// === AST ===

impl SpanTreeGenerator for Ast {
    fn generate_node(&self, ctx:Context) -> FallibleResult<Node> {
        use ast::known::*;

        if let Some(infix) = GeneralizedInfix::try_new(self) {
            infix.flatten().generate_node(ctx)
        } else {
            match self.shape() {
                ast::Shape::Prefix {..} =>
                    ast::prefix::Chain::try_new(self).unwrap().generate_node(ctx),
                // TODO[a] add other shapes, e.g. macros
                _  => Ok(Node {
                    size     : Size::new(self.len()),
                    children : default(),
                    kind     : ast_node_kind(false,&ctx),
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

impl SpanTreeGenerator for ast::opr::Chain {
    fn generate_node(&self, ctx: Context) -> FallibleResult<Node> {
        let target_ctx      = Context::Operator(&self.operator.name);
        let opr_ctx         = Context::Operator(&self.operator.name);
        let node_and_offset = match &self.target {
            Some(sast) => sast.generate_node(target_ctx).map(|n| (n,sast.off)),
            None       => Ok((Node::new_empty(),0)),
        };

        let (node,_) = self.args.iter().enumerate().fold(node_and_offset, |(result),(i,elem)| {
            let (node,off) = result?;
            let is_first  = i == 0;
            let is_last   = i + 1 == self.args.len();
            let is_target = is_first && node.kind != node::Kind::Empty;
            let opr_ast  = Located::new(elem.crumb_to_operator(),elem.operator.ast().clone_ref());

            let mut gen  = ChildGenerator::default();
            if is_target { gen.generate_empty_node(); }
            gen.add_node(vec![elem.crumb_to_previous()],node);
            if is_target { gen.generate_empty_node(); }
            gen.spacing(off);
            gen.generate_ast_node(opr_ast,opr_ctx)?;
            if let Some(sast) = &elem.operand {
                let arg_ast = Located::new(elem.crumb_to_operand(), sast.wrapped.clone_ref());
                gen.spacing(sast.off);
                gen.generate_ast_node(arg_ast, Context::Argument)?;
            }
            gen.generate_empty_node();

            if ast::opr::assoc(&self.operator) == Assoc::Right {
                gen.reverse_children();
            }

            Ok((Node {
                kind: ast_node_kind(!is_last,&ctx),
                size: gen.current_offset,
                children: gen.children,
            }, elem.offset))
        })?;
        Ok(node)
    }
}


// === Application ===

impl SpanTreeGenerator for ast::prefix::Chain {
    fn generate_node(&self, ctx: Context) -> FallibleResult<Node> {
        use ast::crumbs::PrefixCrumb::*;
        let node = self.func.generate_node(Context::PrefixFunc);
        self.args.iter().enumerate().fold(node, |node,(i,arg)| {
            let node     = node?;
            let is_first = i == 0;
            let is_last  = i + 1 == self.args.len();
            let arg_ctx  = if is_first {Context::PrefixTarget} else {Context::Argument};

            let mut gen = ChildGenerator::default();
            gen.add_node(vec![Func.into()],node);
            if is_first { gen.generate_empty_node(); }
            gen.spacing(arg.off);
            gen.generate_ast_node(Located::new(Arg,arg.wrapped.clone_ref()),arg_ctx)?;
            Ok(Node {
                kind     : ast_node_kind(!is_last,&ctx),
                size     : gen.current_offset,
                children : gen.children,
            })
        })
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
            .add_empty_child(0)
            .add_child(0,11,Target,vec![InfixCrumb::LeftOperand])
                .add_empty_child(0)
                .add_leaf (0,1,Target   ,vec![InfixCrumb::LeftOperand])
                .add_empty_child(1)
                .add_leaf (2,1,Operation,vec![InfixCrumb::Operator])
                .add_child(4,7,Argument ,vec![InfixCrumb::RightOperand])
                    .add_leaf(0,3,Operation,vec![PrefixCrumb::Func])
                    .add_empty_child(3)
                    .add_leaf(4,3,Target   ,vec![PrefixCrumb::Arg])
                    .add_empty_child(7)
                    .done()
                .add_empty_child(11)
                .done()
            .add_empty_child(11)
            .add_leaf(12,1,Operation,vec![InfixCrumb::Operator])
            .add_leaf(14,1,Argument,vec![InfixCrumb::RightOperand])
            .add_empty_child(15)
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
                    .add_empty_child(0)
                    .add_leaf(0,1,Target,   vec![InfixCrumb::LeftOperand])
                    .add_empty_child(1)
                    .add_leaf(2,1,Operation,vec![InfixCrumb::Operator])
                    .add_leaf(4,1,Argument ,vec![InfixCrumb::RightOperand])
                    .add_empty_child(5)
                    .done()
                .add_leaf (6,1 ,Operation,vec![InfixCrumb::Operator])
                .add_child(8,14,Argument ,vec![InfixCrumb::RightOperand])
                    .add_child(0,11,Chained,vec![PrefixCrumb::Func])
                        .add_child(0,7,Chained,vec![PrefixCrumb::Func])
                            .add_leaf(0,3,Operation,vec![PrefixCrumb::Func])
                            .add_empty_child(3)
                            .add_leaf(4,3,Target   ,vec![PrefixCrumb::Arg])
                            .add_empty_child(7)
                            .done()
                        .add_leaf(8,3,Argument,vec![PrefixCrumb::Arg])
                        .add_empty_child(11)
                        .done()
                    .add_leaf(12,2,Argument,vec![PrefixCrumb::Arg])
                    .add_empty_child(14)
                    .done()
                .add_empty_child(22)
                .done()
            .add_leaf(23,1,Operation,vec![InfixCrumb::Operator])
            .add_leaf(25,1,Argument ,vec![InfixCrumb::RightOperand])
            .add_empty_child(26)
            .build();

        assert_eq!(expected,tree);
    }

    #[wasm_bindgen_test]
    fn generating_span_tree_from_right_assoc_operator() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line("1,2,3").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = TreeBuilder::new(5)
            .add_empty_child(0)
            .add_leaf (0,1,Argument ,vec![InfixCrumb::LeftOperand])
            .add_leaf (1,1,Operation,vec![InfixCrumb::Operator])
            .add_child(2,3,Chained  ,vec![InfixCrumb::RightOperand])
                .add_empty_child(0)
                .add_leaf(0,1,Argument ,vec![InfixCrumb::LeftOperand])
                .add_leaf(1,1,Operation,vec![InfixCrumb::Operator])
                .add_empty_child(2)
                .add_leaf(2,1,Target   ,vec![InfixCrumb::RightOperand])
                .add_empty_child(3)
                .done()
            .build();

        assert_eq!(expected,tree)
    }

    #[wasm_bindgen_test]
    fn generating_span_tree_from_section() {
        let parser = Parser::new_or_panic();
        // The star makes `SectionSides` ast being one of the parameters of + chain. First + makes
        // SectionRight, and last + makes SectionLeft.
        let ast    = parser.parse_line("+ * + + 2 +").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = TreeBuilder::new(11)
            .add_child(0,9,Chained,vec![SectionLeftCrumb::Arg])
                .add_child(0,5,Chained,vec![InfixCrumb::LeftOperand])
                    .add_child(0,3,Chained,vec![SectionLeftCrumb::Arg])
                        .add_empty_child(0)
                        .add_leaf (0,1,Operation,vec![SectionRightCrumb::Opr])
                        .add_child(2,1,Argument ,vec![SectionRightCrumb::Arg])
                            .add_empty_child(0)
                            .add_leaf(0,1,Operation,vec![SectionSidesCrumb])
                            .add_empty_child(1)
                            .done()
                        .add_empty_child(3)
                        .done()
                    .add_leaf(4,1,Operation,vec![SectionLeftCrumb::Opr])
                    .add_empty_child(5)
                    .done()
                .add_leaf(6,1,Operation,vec![InfixCrumb::Operator])
                .add_leaf(8,1,Argument ,vec![InfixCrumb::RightOperand])
                .add_empty_child(9)
                .done()
            .add_leaf(8,1,Operation,vec![SectionLeftCrumb::Opr])
            .add_empty_child(9)
            .build();

        assert_eq!(expected,tree);
    }

    #[wasm_bindgen_test]
    fn generating_span_tree_from_right_assoc_section() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line(",2,").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = TreeBuilder::new(3)
            .add_empty_child(0)
            .add_leaf (0,1,Operation,vec![SectionRightCrumb::Opr])
            .add_child(1,2,Chained  ,vec![SectionRightCrumb::Arg])
                .add_empty_child(0)
                .add_leaf(0,1,Argument ,vec![SectionLeftCrumb::Arg])
                .add_leaf(1,1,Operation,vec![SectionLeftCrumb::Opr])
                .add_empty_child(2)
                .done()
            .build();

        assert_eq!(expected,tree);
    }
}
