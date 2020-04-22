//! Utilities for dealing with operators and Ast nodes related to them, like `Infix`, `Section*`.

use crate::prelude::*;

use crate::Infix;
use crate::SectionLeft;
use crate::SectionRight;
use crate::SectionSides;
use crate::Ast;
use crate::Shifted;
use crate::Shape;
use crate::assoc::Assoc;
use crate::crumbs::{Crumb, InfixCrumb, SectionLeftCrumb, SectionRightCrumb};
use crate::crumbs::Located;
use crate::known;
use utils::vec::VecExt;


/// Identifiers of operators with special meaning for IDE.
pub mod predefined {
    /// Used to create type paths (like `Int.+` or `IO.println`).
    pub const ACCESS : &str = ".";
    /// Used to create bindings, e.g. `add a b = a + b` or `foo = 5`.
    pub const ASSIGNMENT : &str = "=";
    /// Used to create lambda expressions, e.g. `a -> b -> a + b`.
    pub const ARROW : &str = "->";
}

/// Checks if the given AST has Opr shape with the name matching given string.
pub fn is_opr_named(ast:&Ast, name:impl Str) -> bool {
    let opr_opt = known::Opr::try_from(ast).ok();
    opr_opt.contains_if(|opr| opr.name == name.as_ref())
}

/// Checks if given Ast is an assignment operator identifier.
pub fn is_assignment_opr(ast:&Ast) -> bool {
    is_opr_named(ast,predefined::ASSIGNMENT)
}

/// Checks if given Ast is an assignment operator identifier.
pub fn is_arrow_opr(ast:&Ast) -> bool {
    is_opr_named(ast,predefined::ARROW)
}

/// If given Ast is a specific infix operator application, returns it.
pub fn to_specific_infix(ast:&Ast, name:&str) -> Option<known::Infix> {
    let infix = known::Infix::try_from(ast).ok()?;
    is_opr_named(&infix.opr,name).then(infix)
}

/// If given Ast is an assignment infix expression, returns it as Some known::Infix.
pub fn to_assignment(ast:&Ast) -> Option<known::Infix> {
    to_specific_infix(ast,predefined::ASSIGNMENT)
}

/// If given Ast is an arrow infix expression, returns it as Some known::Infix.
pub fn to_arrow(ast:&Ast) -> Option<known::Infix> {
    to_specific_infix(ast,predefined::ARROW)
}

/// Checks if a given node is an assignment infix expression.
pub fn is_assignment(ast:&Ast) -> bool {
    let infix = known::Infix::try_from(ast);
    infix.map(|infix| is_assignment_opr(&infix.opr)).unwrap_or(false)
}



// ===========================
// === Chain-related types ===
// ===========================

/// Infix operator operand. Optional, as we deal with Section* nodes as well.
pub type Operand = Option<Shifted<Ast>>;

/// Infix operator standing between (optional) operands.
pub type Operator = known::Opr;

/// Creates `Operand` from `ast` with position relative to the given `parent` node.
pub fn make_operand(ast:Ast, off:usize) -> Operand {
    let wrapped = ast;
    Some(Shifted{wrapped,off})
}

/// Creates `Operator` from `ast` with position relative to the given `parent` node.
pub fn make_operator(opr:&Ast) -> Option<Operator> {
    known::Opr::try_from(opr).ok()
}

/// Describes associativity of the given operator AST.
fn assoc(ast:&known::Opr) -> Assoc {
    Assoc::of(&ast.name)
}



// ========================
// === GeneralizedInfix ===
// ========================

/// An abstraction over `Infix` and all `SectionSth` nodes. Stores crumb locations for all its ASTs.
#[derive(Clone,Debug)]
pub struct GeneralizedInfix {
    /// Left operand, if present.
    pub left  : Operand,
    /// The operator, always present.
    pub opr   : Operator,
    /// Right operand, if present.
    pub right : Operand,
}

impl GeneralizedInfix {
    /// Tries interpret given AST node as GeneralizedInfix. Returns None, if Ast is not any kind of
    /// application on infix operator.
    pub fn try_new(ast:&Ast) -> Option<GeneralizedInfix> {
        match ast.shape().clone() {
            Shape::Infix(infix) => Some(GeneralizedInfix{
                left  : make_operand (infix.larg,infix.loff),
                opr   : make_operator(&infix.opr)?,
                right : make_operand (infix.rarg,infix.roff),
            }),
            Shape::SectionLeft(left) => Some(GeneralizedInfix{
                left  : make_operand (left.arg,left.off),
                opr   : make_operator(&left.opr)?,
                right : None,
            }),
            Shape::SectionRight(right) => Some(GeneralizedInfix{
                left  : None,
                opr   : make_operator(&right.opr)?,
                right : make_operand (right.arg,right.off),
            }),
            Shape::SectionSides(sides) => Some(GeneralizedInfix{
                left  : None,
                opr   : make_operator(&sides.opr)?,
                right : None,
            }),
            _ => None,
        }
    }

    pub fn new_from_operands(target:Operand, opr:Operator, argument:Operand) -> Self {
        match assoc(&opr) {
            Assoc::Left => GeneralizedInfix {opr,
                left  : target,
                right : argument,
            },
            Assoc::Right => GeneralizedInfix {opr,
                left  : argument,
                right : target,
            },
        }
    }

    /// Convert to AST node.
    pub fn into_ast(self) -> Ast {
        match (self.left,self.right) {
            (Some(left),Some(right)) => Infix{
                larg : left.wrapped,
                loff : left.off,
                opr  : self.opr.into(),
                roff : right.off,
                rarg : right.wrapped,
            }.into(),
            (Some(left),None) => SectionLeft {
                arg : left.wrapped,
                off : left.off,
                opr : self.opr.into(),
            }.into(),
            (None,Some(right)) => SectionRight {
                opr : self.opr.into(),
                off : right.off,
                arg : right.wrapped,
            }.into(),
            (None,None) => SectionSides {
                opr : self.opr.into()
            }.into()
        }
    }

    /// Associativity of the operator used in this infix expression.
    pub fn assoc(&self) -> Assoc {
        assoc(&self.opr)
    }

    /// Identifier name  of the operator used in this infix expression.
    pub fn name(&self) -> &str {
        &self.opr.name
    }

    /// The self operand, target of the application.
    pub fn target_operand(&self) -> Operand {
        match self.assoc() {
            Assoc::Left  => self.left.clone(),
            Assoc::Right => self.right.clone(),
        }
    }

    /// Operand other than self.
    pub fn argument_operand(&self) -> Operand {
        match self.assoc() {
            Assoc::Left  => self.right.clone(),
            Assoc::Right => self.left.clone(),
        }
    }

    /// Converts chain of infix applications using the same operator into `Chain`.
    /// Sample inputs are `x,y,x` or `a+b+` or `+5+5+5`. Note that `Sides*` nodes
    /// are also supported, along the `Infix` nodes.
    pub fn flatten(&self) -> Chain {
        self.flatten_with_offset(0)
    }

    fn flatten_with_offset(&self, offset:usize) -> Chain {
        let target = self.target_operand();
        let rest   = ChainElement {offset,
            operator : self.opr.clone(),
            operand  : self.argument_operand(),
        };

        let target_subtree_infix = target.clone().and_then(|sast| {
            let off = sast.off;
            GeneralizedInfix::try_new(&sast.wrapped).map(|wrapped| Shifted{wrapped,off})
        });
        let mut target_subtree_flat = match target_subtree_infix {
            Some(target_infix) if target_infix.name() == self.name() =>
                target_infix.flatten_with_offset(target_infix.off),
            _ => Chain { target, args:Vec::new(), operator:self.opr.clone() },
        };

        target_subtree_flat.args.push(rest);
        target_subtree_flat
    }
}



// =============
// === Chain ===
// =============

/// Result of flattening infix operator chain, like `a+b+c` or `Foo.Bar.Baz`.
#[derive(Clone,Debug)]
pub struct Chain {
    /// The primary application target (left- or right-most operand, depending on
    /// operators associativity).
    pub target : Operand,
    /// Subsequent operands applied to the `target`.
    pub args   : Vec<ChainElement>,
    /// Operator AST. Generally all operators in the chain should be the same (except for id).
    /// It is not specified which exactly operator's in the chain this AST belongs to.
    pub operator : known::Opr,
}

impl Chain {
    /// If this is infix, it flattens whole chain and returns result.
    /// Otherwise, returns None.
    pub fn try_new(ast:&Ast) -> Option<Chain> {
        GeneralizedInfix::try_new(&ast).map(|infix| infix.flatten())
    }

    /// Flattens infix chain if this is infix application of given operator.
    pub fn try_new_of(ast:&Ast, operator:&str) -> Option<Chain> {
        let infix = GeneralizedInfix::try_new(&ast)?;
        (infix.name() == operator).as_some_from(|| infix.flatten())
    }

    /// Iterates over &Located<Ast>, beginning with target (this argument) and then subsequent
    /// arguments.
    pub fn enumerate_operands<'a>(&'a self) -> impl Iterator<Item=Located<&'a Shifted<Ast>>> + 'a {
        let this_crumbs = self.args.iter().rev().map(ChainElement::crumb_to_previous).collect_vec();
        let this        = self.target.as_ref().map(|opr| Located::new(this_crumbs,opr));
        let args        = self.args.iter().enumerate().map(move |(i,elem)| elem.operand.as_ref().map(|opr| {
            let to_infix = self.args.iter().skip(i+1).rev().map(ChainElement::crumb_to_previous);
            let crumbs   = to_infix.chain(elem.crumb_to_operand()).collect_vec();
            Located::new(crumbs,opr)
        }));
        std::iter::once(this).chain(args).flatten()
    }

    pub fn enumerate_operators<'a>(&'a self) -> impl Iterator<Item=Located<&'a known::Opr>> + 'a {
        self.args.iter().enumerate().map(move |(i,elem)| {
            let to_infix = self.args.iter().skip(i+1).rev().map(ChainElement::crumb_to_previous);
            let crumbs   = to_infix.chain(elem.crumb_to_operator()).collect_vec();
            Located::new(crumbs,&elem.operator)
        })
    }

    pub fn insert_operand(&mut self, at_index:usize, operand:Shifted<Ast>) {
        let mut operand = Some(operand);
        let operator    = chain.operator.clone_ref();
        let offset      = operand.off;
        if at_index == 0 {
            std::mem::swap(&mut operand, &mut self.target);
            self.args.push_front(ChainElement{operator,operand,offset})
        } else {
            self.args.insert(at_index-1,ChainElement{operator,operand,offset})
        }
    }

    pub fn push_operand(&mut self, operand:Shifted<Ast>) {
        let last_index = self.args.len() + 1;
        self.insert_operand(last_index,operand)
    }

    pub fn push_front_operand(&mut self, operand:Shifted<Ast>) {
        self.insert_operand(0,operand)
    }

    pub fn fold_arg(&mut self) {
        if let Some(element) = self.args.pop_front() {
            let target    = std::mem::take(&mut self.target);
            let new_infix = GeneralizedInfix::new_from_operands(target,element.operator,element.operand);
            let new_shifted = Shifted {
                wrapped : new_infix.into_ast(),
                off     : element.offset,
            };
            self.target = Some(new_shifted)
        }
    }

    pub fn into_ast(mut self) -> Ast {
        while !self.args.is_empty() {
            self.fold_arg()
        }
        // TODO[ao] the only case when target is none is when chain have no target and no arguments.
        // But perhaps someone could thing that this is a valid chain. To consider returning error
        // here.
        self.target.unwrap().wrapped
    }
}

/// Element of the infix application chain, i.e. operator and its operand.
#[derive(Clone,Debug)]
pub struct ChainElement {
    #[allow(missing_docs)]
    pub operator : Operator,
    /// Operand on the opposite side to `this` argument.
    /// Depending on operator's associativity it is either right (for left-associative operators)
    /// or on the left side of operator.
    pub operand : Operand,
    /// Offset between this operand and the next operator.
    pub offset : usize,
}

impl ChainElement {
    pub fn crumb_to_previous(&self) -> Crumb {
        let has_operand = self.operand.is_some();
        match assoc(&self.operator) {
            Assoc::Left if has_operand  => InfixCrumb::LeftOperand.into(),
            Assoc::Left                 => SectionLeftCrumb::Arg.into(),
            Assoc::Right if has_operand => InfixCrumb::RightOperand.into(),
            Assoc::Right                => SectionRightCrumb::Arg.into(),
        }
    }

    pub fn crumb_to_operand(&self) -> Crumb {
        match assoc(&self.operator) {
            Assoc::Left  => InfixCrumb::RightOperand.into(),
            Assoc::Right => InfixCrumb::LeftOperand.into(),
        }
    }

    pub fn crumb_to_operator(&self) -> Crumb {
        if self.operand.is_some() {
            InfixCrumb::Operator.into()
        } else {
            match assoc(&self.operator) {
                Assoc::Left  => SectionLeftCrumb::Opr.into(),
                Assoc::Right => SectionRightCrumb::Opr.into(),
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn expect_at(operand:&Operand, expected_ast:&Ast) {
        assert_eq!(&operand.as_ref().unwrap().wrapped,expected_ast);
    }

    fn test_enumerating(chain:&Chain, root_ast:&Ast, expected_asts:&[&Ast]) {
        assert_eq!(chain.enumerate_operands().count(), expected_asts.len());
        for (elem,expected) in chain.enumerate_operands().zip(expected_asts) {
            assert_eq!(elem.item.wrapped,**expected);
            let ast = root_ast.get_traversing(elem.crumbs).unwrap();
            assert_eq!(ast,*expected);
        }
    }

    #[test]
    fn infix_chain_tests() {
        let a               = Ast::var("a");
        let b               = Ast::var("b");
        let c               = Ast::var("c");
        let a_plus_b        = Ast::infix(a.clone(),"+",b.clone());
        let a_plus_b_plus_c = Ast::infix(a_plus_b.clone(),"+",c.clone());
        let mut chain       = Chain::try_new(&a_plus_b_plus_c).unwrap();
        expect_at(&chain.target,&a);
        expect_at(&chain.args[0].operand,&b);
        expect_at(&chain.args[1].operand,&c);

        test_enumerating(&chain,&a_plus_b_plus_c, &[&a,&b,&c]);
    }

    #[test]
    fn infix_chain_tests_right() {
        let a                 = Ast::var("a");
        let b                 = Ast::var("b");
        let c                 = Ast::var("c");
        let b_comma_c         = Ast::infix(b.clone(),",",c.clone());
        let a_comma_b_comma_c = Ast::infix(a.clone(),",",b_comma_c.clone());
        let chain             = Chain::try_new(&a_comma_b_comma_c).unwrap();
        expect_at(&chain.target,&c);
        expect_at(&chain.args[0].operand,&b);
        expect_at(&chain.args[1].operand,&a);
    }
}
