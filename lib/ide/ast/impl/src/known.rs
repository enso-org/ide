//! This module provides KnownAst<T> wrapper over Ast that allows expressing that we already
//! know what `Shape` variant is being stored within this `Ast` node.

use crate::prelude::*;

use crate::Ast;
use crate::Shape;



// =================
// === Known AST ===
// =================

/// Wrapper for an AST node of known shape type that we can access.
/// Use `TryFrom<&Ast>` to obtain values.
///
/// Provides `Deref` implementation that allows accessing underlying shape `T` value.
#[derive(Derivative)]
#[derivative(Clone(bound=""))]
#[derive(Debug)]
pub struct KnownAst<T>(Ast, PhantomData<T>);

impl<T> KnownAst<T> {
    /// Checks if the shape of given Ast node is compatible with `T`.
    /// If yes, returns Ok with Ast node wrapped as KnownAst.
    /// Otherwise, returns an error.
    pub fn try_new<E>(ast:Ast) -> Result<KnownAst<T>,E>
    where for<'t> &'t Shape<Ast>: TryInto<&'t T, Error=E> {
        if let Some(error_matching) = ast.shape().try_into().err() {
            Err(error_matching)
        } else {
            Ok(KnownAst(ast,default()))
        }
    }
}

impl<T,E> Deref for KnownAst<T>
where for<'t> &'t Shape<Ast> : TryInto<&'t T,Error=E>,
                           E : Debug, {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let result = self.0.shape().try_into();
        // Below must never happen, as the only function for constructing values does check
        // if the shape type matches the `T`.
        result.expect("Internal Error: wrong shape in KnownAst.")
    }
}

impl<T,E> TryFrom<&Ast> for KnownAst<T>
where for<'t> &'t Shape<Ast>:TryInto<&'t T,Error=E> {
    type Error = E;
    fn try_from(ast:&Ast) -> Result<KnownAst<T>,Self::Error> {
        KnownAst::try_new(ast.clone())
    }
}

impl<T,E> TryFrom<Ast> for KnownAst<T>
where for<'t> &'t Shape<Ast>:TryInto<&'t T,Error=E> {
    type Error = E;
    fn try_from(ast:Ast) -> Result<KnownAst<T>,Self::Error> {
        KnownAst::try_new(ast)
    }
}

/// One can always throw away the knowledge.
impl<T> From<KnownAst<T>> for Ast {
    fn from(known_ast:KnownAst<T>) -> Ast {
        known_ast.0
    }
}


// ===============
// === Aliases ===
// ===============

pub type Unrecognized  = KnownAst<crate::Unrecognized>;
pub type InvalidQuote  = KnownAst<crate::InvalidQuote>;
pub type InlineBlock   = KnownAst<crate::InlineBlock>;
pub type Blank         = KnownAst<crate::Blank>;
pub type Var           = KnownAst<crate::Var>;
pub type Cons          = KnownAst<crate::Cons>;
pub type Opr           = KnownAst<crate::Opr>;
pub type Mod           = KnownAst<crate::Mod>;
pub type InvalidSuffix = KnownAst<crate::InvalidSuffix<Ast>>;
pub type Number        = KnownAst<crate::Number>;
pub type DanglingBase  = KnownAst<crate::DanglingBase>;
pub type TextLineRaw   = KnownAst<crate::TextLineRaw>;
pub type TextLineFmt   = KnownAst<crate::TextLineFmt<Ast>>;
pub type TextBlockRaw  = KnownAst<crate::TextBlockRaw>;
pub type TextBlockFmt  = KnownAst<crate::TextBlockFmt<Ast>>;
pub type TextUnclosed  = KnownAst<crate::TextUnclosed<Ast>>;
pub type Prefix        = KnownAst<crate::Prefix<Ast>>;
pub type Infix         = KnownAst<crate::Infix<Ast>>;
pub type SectionLeft   = KnownAst<crate::SectionLeft<Ast>>;
pub type SectionRight  = KnownAst<crate::SectionRight<Ast>>;
pub type SectionSides  = KnownAst<crate::SectionSides<Ast>>;
pub type Module        = KnownAst<crate::Module<Ast>>;
pub type Block         = KnownAst<crate::Block<Ast>>;
pub type Match         = KnownAst<crate::Match<Ast>>;
pub type Ambiguous     = KnownAst<crate::Ambiguous>;



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let ast_var = crate::Ast::var("foo");
        // This is truly var, so we can unwrap and directly access it's fields.
        let known_var = Var::try_from(&ast_var).unwrap();
        assert_eq!(known_var.name, "foo");

        let known_var: Var = ast_var.clone().try_into().unwrap();
        assert_eq!(known_var.name, "foo");


        // This is not an Infix, so we won't get KnownAst object.
        let known_infix_opt = Infix::try_from(&ast_var);
        assert!(known_infix_opt.is_err());
    }
}
