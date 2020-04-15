use crate::prelude::*;

use crate::Type;
use crate::Node;
use crate::tree;

use ast::Ast;
use ast::HasLength;
use data::text::Size;
use ast::assoc::Assoc;
use ast::opr::GeneralizedInfix;
use ast::crumbs::Located;


// =============
// === Trait ===
// =============

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum ChainingContext<'a> {
    None, Prefix, Operator(&'a str),
}

pub trait SpanTreeGenerator {
    fn generate_node(&self, ctx:ChainingContext) -> FallibleResult<Node>;

    fn generate_tree(&self) -> FallibleResult<Node> {
        self.generate_node(ChainingContext::None)
    }
}


// =================
// === Utilities ===
// =================

// === Child Generator ===

#[derive(Debug,Default)]
struct ChildGenerator {
    current_offset : Size,
    children       : Vec<tree::Child>,
}

impl ChildGenerator {
    fn spacing(&mut self, size:usize) {
        self.current_offset += Size::new(size);
    }

    fn generate_ast_node
    (&mut self, child_ast:Located<Ast>, ctx:ChainingContext)
    -> FallibleResult<&tree::Child> {
        let child = tree::Child {
            node                : child_ast.item.generate_node(ctx)?,
            offset              : self.current_offset,
            chained_with_parent : ast_can_be_chained_with_parent(&child_ast,ctx),
            ast_crumbs          : child_ast.crumbs
        };
        self.current_offset += child.node.len;
        self.children.push(child);
        Ok(self.children.last().unwrap())
    }

    fn generate_empty_node(&mut self) -> &tree::Child {
        let child = tree::Child {
            node                : Node::new_empty(),
            offset              : self.current_offset,
            chained_with_parent : false,
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
    fn generate_node(&self, ctx:ChainingContext) -> FallibleResult<Node> {
        use ast::known::*;

        if let Some(infix) = GeneralizedInfix::try_new_root(self) {
            infix.generate_node(ctx)
        } else {
            match self.shape() {
                ast::Shape::Prefix {..} =>
                    Prefix::try_new(self.clone_ref()).unwrap().generate_node(ctx),
                _  => Ok(Node {
                    len       : Size::new(self.len()),
                    children  : default(),
                    node_type : crate::Type::Ast,
                }),
            }
        }
    }
}


// === Operators (Sections and Infixes) ===

impl SpanTreeGenerator for GeneralizedInfix {

    fn generate_node(&self, ctx:ChainingContext) -> FallibleResult<Node> {
        let have_empty = !infix_can_be_chained_with_parent(self,ctx);
        let assoc      = self.assoc();
        let target_ctx = ChainingContext::Operator(&self.opr.name);

        let (left_empty,left_ctx,right_empty,right_ctx) = match assoc {
            Assoc::Left  => (false     , target_ctx           , have_empty, ChainingContext::None),
            Assoc::Right => (have_empty, ChainingContext::None, false     , target_ctx),
        };

        let mut gen = ChildGenerator::default();
        match &self.left {
            Some(arg) => {
                if left_empty {
                    gen.generate_empty_node();
                }
                gen.generate_ast_node(arg.arg.clone(),left_ctx)?;
                gen.spacing(arg.offset);
            }
            None => { gen.generate_empty_node(); },
        }
        gen.generate_ast_node(self.opr.clone().map(|opr| opr.ast().clone_ref()),ChainingContext::None)?;
        match &self.right {
            Some(arg) => {
                gen.spacing(arg.offset);
                gen.generate_ast_node(arg.arg.clone(),right_ctx)?;
                if right_empty {
                    gen.generate_empty_node();
                }
            }
            None => { gen.generate_empty_node(); },
        }
        Ok(Node {
            node_type : Type::Ast,
            len       : gen.current_offset,
            children  : gen.children,
        })
    }
}



// === Application ===

impl SpanTreeGenerator for ast::known::Prefix {

    fn generate_node(&self, ctx: ChainingContext) -> FallibleResult<Node> {
        let should_have_empty = !prefix_can_be_chained_with_parent(ctx);

        use ast::crumbs::PrefixCrumb::*;
        let mut gen = ChildGenerator::default();
        gen.generate_ast_node(Located::new(vec![Func],self.func.clone_ref()),ChainingContext::Prefix)?;
        gen.spacing(self.off);
        gen.generate_ast_node(Located::new(vec![Arg],self.arg.clone_ref()),ChainingContext::None)?;
        if should_have_empty {
            gen.generate_empty_node();
        }
        Ok(Node {
            node_type: Type::Ast,
            len: Size::new(self.len()),
            children: gen.children,
        })
    }
}



// ===========================
// === Chaining Conditions ===
// ===========================

fn ast_can_be_chained_with_parent(ast:&Ast, ctx:ChainingContext) -> bool {
    if let Some(infix) = GeneralizedInfix::try_new(&Located::new_root(ast.clone_ref())) {
        infix_can_be_chained_with_parent(&infix,ctx)
    } else {
        match ast.shape() {
            ast::Shape::Prefix {..} => prefix_can_be_chained_with_parent(ctx),
            _                       => false,
        }
    }
}

fn infix_can_be_chained_with_parent(infix:&GeneralizedInfix, ctx:ChainingContext) -> bool {
    match ctx {
        ChainingContext::Operator(name) if infix.opr.item.name == *name => true,
        _                                                               => false,
    }
}

fn prefix_can_be_chained_with_parent(ctx:ChainingContext) -> bool {
    match ctx {
        ChainingContext::Prefix => true,
        _                       => false,
    }
}


// ============
// === Test ===
// ============

#[cfg(test)]
mod test {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;
    use parser::Parser;
    use crate::builder::{RootBuilder, Builder};
    use ast::crumbs::{InfixCrumb, PrefixCrumb, SectionLeftCrumb, SectionRightCrumb, SectionSidesCrumb};

    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn generating_span_tree() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line("2 + foo bar - 3").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = RootBuilder::new(15)
            .add_ast_child(0,11,vec![InfixCrumb::LeftOperand])
                .add_ast_leaf(0,1,vec![InfixCrumb::LeftOperand])
                .add_ast_leaf(2,1,vec![InfixCrumb::Operator])
                .add_ast_child(4,7,vec![InfixCrumb::RightOperand])
                    .add_ast_leaf(0,3,vec![PrefixCrumb::Func])
                    .add_ast_leaf(4,3,vec![PrefixCrumb::Arg])
                    .add_empty_child(7)
                    .done()
                .add_empty_child(11)
                .done()
            .add_ast_leaf(12,1,vec![InfixCrumb::Operator])
            .add_ast_leaf(14,1,vec![InfixCrumb::RightOperand])
            .add_empty_child(15)
            .build();

        assert_eq!(expected,tree)
    }

    #[wasm_bindgen_test]
    fn generate_span_tree_with_chains() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line("2 + 3 + foo bar baz + 5").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = RootBuilder::new(23)
            .add_ast_child(0,19,vec![InfixCrumb::LeftOperand])
                .chain_with_parent()
                .add_ast_child(0,5,vec![InfixCrumb::LeftOperand])
                    .chain_with_parent()
                    .add_ast_leaf(0,1,vec![InfixCrumb::LeftOperand])
                    .add_ast_leaf(2,1,vec![InfixCrumb::Operator])
                    .add_ast_leaf(4,1,vec![InfixCrumb::RightOperand])
                    .done()
                .add_ast_leaf(6,1,vec![InfixCrumb::Operator])
                .add_ast_child(8,11,vec![InfixCrumb::RightOperand])
                    .add_ast_child(0,7,vec![PrefixCrumb::Func])
                        .chain_with_parent()
                        .add_ast_leaf(0,3,vec![PrefixCrumb::Func])
                        .add_ast_leaf(4,3,vec![PrefixCrumb::Arg])
                        .done()
                    .add_ast_leaf(8,3,vec![PrefixCrumb::Arg])
                    .add_empty_child(11)
                    .done()
                .done()
            .add_ast_leaf(20,1,vec![InfixCrumb::Operator])
            .add_ast_leaf(22,1,vec![InfixCrumb::RightOperand])
            .add_empty_child(23)
            .build();

        assert_eq!(expected,tree);
    }

    #[wasm_bindgen_test]
    fn generating_span_tree_from_right_assoc_operator() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line("1,2,3").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = RootBuilder::new(5)
            .add_empty_child(0)
            .add_ast_leaf(0,1,vec![InfixCrumb::LeftOperand])
            .add_ast_leaf(1,1,vec![InfixCrumb::Operator])
            .add_ast_child(2,3,vec![InfixCrumb::RightOperand])
                .chain_with_parent()
                .add_ast_leaf(0,1,vec![InfixCrumb::LeftOperand])
                .add_ast_leaf(1,1,vec![InfixCrumb::Operator])
                .add_ast_leaf(2,1,vec![InfixCrumb::RightOperand])
                .done()
            .build();

        assert_eq!(expected,tree)
    }

    #[wasm_bindgen_test]
    fn generating_span_tree_from_section() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line("+ * + 2 +").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = RootBuilder::new(9)
            .add_ast_child(0,7,vec![SectionLeftCrumb::Arg])
                .chain_with_parent()
                .add_ast_child(0,3,vec![InfixCrumb::LeftOperand])
                    .chain_with_parent()
                    .add_empty_child(0)
                    .add_ast_leaf(0,1,vec![SectionRightCrumb::Opr])
                    .add_ast_child(2,1,vec![SectionRightCrumb::Arg])
                        .add_empty_child(0)
                        .add_ast_leaf(0,1,vec![SectionSidesCrumb])
                        .add_empty_child(1)
                        .done()
                    .done()
                .add_ast_leaf(4,1,vec![InfixCrumb::Operator])
                .add_ast_leaf(6,1,vec![InfixCrumb::RightOperand])
                .done()
            .add_ast_leaf(8,1,vec![SectionLeftCrumb::Opr])
            .add_empty_child(9)
            .build();

        assert_eq!(expected,tree);
    }

    #[wasm_bindgen_test]
    fn generating_span_tree_from_right_assoc_section() {
        let parser = Parser::new_or_panic();
        let ast    = parser.parse_line(",2").unwrap();
        let tree   = ast.generate_tree().unwrap();

        let expected = RootBuilder::new(2)
            .add_empty_child(0)
            .add_ast_leaf(0,1,vec![SectionRightCrumb::Opr])
            .add_ast_leaf(1,1,vec![SectionRightCrumb::Arg])
            .build();

        assert_eq!(expected,tree);
    }
}
