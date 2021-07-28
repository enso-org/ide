//! A module with all functions used to synchronize different representations of our language
//! module.

use crate::prelude::*;

use ast::{Ast, opr, prefix, known};
use crate::double_representation::definition::{ScopeKind, DefinitionName, DefinitionInfo};
use ast::crumbs::{Located, InfixCrumb};

pub mod alias_analysis;
pub mod comment;
pub mod connection;
pub mod definition;
pub mod graph;
pub mod identifier;
pub mod module;
pub mod node;
pub mod refactorings;
pub mod text;
pub mod tp;

#[cfg(test)]
pub mod test_utils;


// ==============
// === Consts ===
// ==============

/// Indentation value from language specification:
///
/// Indentation: Indentation is four spaces, and all tabs are converted to 4 spaces. This is not
/// configurable on purpose.
///
/// Link: https://github.com/enso-org/enso/blob/main/doc/syntax/encoding.md
pub const INDENT : usize = 4;

pub enum LineKind {
    Definition {
        ast  : known::Infix,
        name : Located<DefinitionName>,
        args : Vec<Located<Ast>>,
    },
    ExpressionAssignment {
        ast : known::Infix,
    },
    ExpressionPlain {
        ast : Ast,
    },
}

pub fn discern_line
(ast:&Ast, kind:ScopeKind) -> Option<LineKind> {
    use LineKind::*;
    let infix = match opr::to_assignment(ast) {
        Some(infix) => infix,
        None        => {
            return if ast::macros::is_documentation_comment(ast) {
                None
            } else {
                Some(ExpressionPlain {ast:ast.clone_ref()})
            }
        }
    };
    // There two cases - function name is either a Var or operator.
    // If this is a Var, we have Var, optionally under a Prefix chain with args.
    // If this is an operator, we have SectionRight with (if any prefix in arguments).
    let lhs  = Located::new(InfixCrumb::LeftOperand,prefix::Chain::from_ast_non_strict(&infix.larg));
    let name = lhs.entered(|chain| {
        let name_ast = chain.located_func();
        name_ast.map(DefinitionName::from_ast)
    }).into_opt()?;
    let args = lhs.enumerate_args().map(|located_ast| {
        // We already in the left side of assignment, so we need to prepend this crumb.
        let left   = std::iter::once(ast::crumbs::Crumb::from(InfixCrumb::LeftOperand));
        let crumbs = left.chain(located_ast.crumbs);
        let ast    = located_ast.item.clone();
        Located::new(crumbs,ast)
    }).collect_vec();

    // Note [Scope Differences]
    if kind == ScopeKind::NonRoot {
        // 1. Not an extension method but an old setter syntax. Currently not supported in the
        // language, treated as node with invalid pattern.
        let is_setter = !name.extended_target.is_empty();
        // 2. No explicit args -- this is a proper node, not a definition.
        let is_node = args.is_empty();
        if is_setter || is_node {
            return Some(ExpressionAssignment{ast:infix})
        }
    };

    Some(LineKind::Definition {
        args,name,ast:infix.clone_ref()
    })
}
