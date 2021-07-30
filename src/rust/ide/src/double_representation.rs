//! A module with all functions used to synchronize different representations of our language
//! module.

use crate::prelude::*;

use ast::{Ast, opr, prefix, known};
use crate::double_representation::definition::DefinitionName;
use crate::double_representation::definition::ScopeKind;
use ast::crumbs::{Located, InfixCrumb};
use ast::macros::DocCommentInfo;

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

/// What kind of node or definition a line should be treated as.
#[derive(Clone,Debug)]
pub enum LineKind {
    /// Definition is a binding, which defines a new entity with arguments.
    Definition {
        /// Ast of the whole binding.
        ast  : known::Infix,
        /// Definition name. If this is an extension method, it might imply an implicit argument.
        name : Located<DefinitionName>,
        /// Explicit arguments. Note that this might be empty when there are implicit arguments.
        args : Vec<Located<Ast>>,
    },
    /// Node in a binding form.
    ExpressionAssignment {
        /// Ast of the whole binding.
        ast : known::Infix,
    },
    /// Node consisting of a plain expression, with no pattern binding.
    ExpressionPlain {
        /// Ast of the whole expression.
        ast : Ast,
    },
    /// Documentation comment lines are not nodes.
    /// Instead, they are discovered and processed as part of nodes that follow them.
    DocumentationComment {
        /// The comment representation.
        comment : DocCommentInfo,
    }
}

impl LineKind {

    /// Tell how the given line (described by Ast) should be treated.
    /// Returns `None` if line does not match any of the expected entities.
    pub fn discern(ast:&Ast, kind:ScopeKind) -> Self {
        use LineKind::*;

        // First of all, if line is not an infix assignment, it can be only node or nothing.
        let infix = match opr::to_assignment(ast) {
            Some(infix) =>
                infix,
            None =>
                return if let Some(comment) = DocCommentInfo::new(ast) {
                    // e.g. `## My comment.`
                    DocumentationComment {comment}
                } else {
                    // The simplest form of node, e.g. `Point 5 10`
                    ExpressionPlain {ast:ast.clone_ref()}
                }
        };

        // Assignment can be either nodes or definitions. To discern, we check the left hand side.
        // For definition it is a prefix chain, where first is the name, then arguments (if explicit).
        // For node it is a pattern, either in a form of Var without args on Cons application.
        let crumb = InfixCrumb::LeftOperand;
        let lhs   = Located::new(crumb,prefix::Chain::from_ast_non_strict(&infix.larg));
        let name  = lhs.entered(|chain| {
            let name_ast = chain.located_func();
            name_ast.map(DefinitionName::from_ast)
        }).into_opt();

        // If this is a pattern match, `name` will fail to construct and we'll treat line as a node.
        // e.g. for `Point x y = get_point …`
        let name = match name {
            Some(name) => name,
            None       => return ExpressionAssignment{ast:infix}
        };

        let args = lhs.enumerate_args().map(|Located{crumbs,item}| {
            // We already in the left side of assignment, so we need to prepend this crumb.
            let crumbs = lhs.crumbs.clone().into_iter().chain(crumbs);
            let ast    = item.clone();
            Located::new(crumbs,ast)
        }).collect_vec();

        // Note [Scope Differences]
        if kind == ScopeKind::NonRoot {
            // 1. Not an extension method but an old setter syntax. Currently not supported in the
            // language, treated as node with invalid pattern.
            // e.g. `point.x = 5`
            let is_setter = !name.extended_target.is_empty();
            // 2. No explicit args -- this is a proper node, not a definition.
            // e.g. `point = Point 5 10`
            let is_node = args.is_empty();
            if is_setter || is_node {
                return ExpressionAssignment{ast:infix}
            }
        };

        Definition {
            args,name,ast:infix.clone_ref()
        }
    }
}

// Note [Scope Differences]
// ========================
// When we are in definition scope (as opposed to global scope) certain patterns should not be
// considered to be function definitions. These are:
// 1. Expressions like "Int.x = …". In module, they'd be treated as extension methods. In
//    definition scope they are treated as invalid constructs (setter syntax in the old design).
// 2. Expression like "foo = 5". In module, this is treated as method definition (with implicit
//    this parameter). In definition, this is just a node (evaluated expression).
