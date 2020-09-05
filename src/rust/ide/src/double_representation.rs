//! A module with all functions used to synchronize different representations of our language
//! module.

pub mod alias_analysis;
pub mod connection;
pub mod definition;
pub mod graph;
pub mod identifier;
pub mod module;
pub mod node;
pub mod refactorings;
pub mod text;

#[cfg(test)]
pub mod test_utils;

use crate::prelude::*;

use crate::double_representation::identifier::Identifier;



// ==============
// === Consts ===
// ==============

/// Indentation value from language specification:
///
/// Indentation: Indentation is four spaces, and all tabs are converted to 4 spaces. This is not
/// configurable on purpose.
///
/// Link: https://github.com/luna/enso/blob/main/doc/syntax/encoding.md
pub const INDENT : usize = 4;


// pub fn target_method_name(ast:&Ast) -> Option<Identifier> {
//     if let Some(chain) = ast::prefix::Chain::from_ast(ast) {
//         target_method_name(&chain.func)
//     } else if let Some(chain) = ast::opr::as_access_chain(ast) {
//         let presumed_name = chain.args.last()?;
//         identifier::Identifier::new(presumed_name.operand.as_ref()?.arg.clone_ref())
//     } else {
//         identifier::Identifier::new(ast.clone())
//     }
// }
