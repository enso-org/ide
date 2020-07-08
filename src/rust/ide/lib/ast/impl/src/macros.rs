//! Utilities for dealing with macro-related parts of AST and language, including `Match` shape and
//! such constructs as lambda expressions.


use crate::prelude::*;

use crate::crumbs::AmbiguousCrumb;
use crate::crumbs::Located;
use crate::crumbs::MatchCrumb;
use crate::known;



// ===============
// === Lambdas ===
// ===============

/// Describes the lambda-expression's three pieces: the argument, the arrow operator and the body.
#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub struct LambdaInfo<'a> {
    pub arg  : Located<&'a Ast>,
    pub opr  : Located<&'a Ast>,
    pub body : Located<&'a Ast>,
}

/// If this is the builtin macro for `->` (lambda expression), returns it as known `Match`.
pub fn as_lambda_match(ast:&Ast) -> Option<known::Match> {
    let macro_match = known::Match::try_from(ast).ok()?;
    let segment     = &macro_match.segs.head;
    crate::opr::is_arrow_opr(&segment.head).then(macro_match)
}

/// Describes the given Ast as lambda, if this is a matched `->` builtin macro.
pub fn as_lambda(ast:&Ast) -> Option<LambdaInfo> {
    let _              = as_lambda_match(ast)?;
    let mut child_iter = ast.iter_subcrumbs();
    let arg            = ast.get_located(child_iter.next()?).ok()?;
    let opr            = ast.get_located(child_iter.next()?).ok()?;
    let body           = ast.get_located(child_iter.next()?).ok()?;
    let is_arrow       = crate::opr::is_arrow_opr(&opr.item);
    is_arrow.then(LambdaInfo {arg,opr,body})
}



// ===================
// === Match Utils ===
// ===================

impl crate::Match<Ast> {
    /// Iterates matched ASTs. Skips segment heads ("keywords").
    /// For example, for `(a)` it iterates only over `a`, skkipping segment heads `(` and `)`.
    pub fn iter_pat_match_subcrumbs<'a>(&'a self) -> impl Iterator<Item=MatchCrumb> + 'a {
        self.iter_subcrumbs().filter(|crumb| {
            use crate::crumbs::SegmentMatchCrumb;
            match crumb {
                MatchCrumb::Segs {val,..} => val != &SegmentMatchCrumb::Head,
                _                         => true
            }
        })
    }
}



// =======================
// === Ambiguous Utils ===
// =======================

impl crate::Ambiguous<Ast> {
    /// Iterates matched ASTs. Skips segment heads ("keywords").
    /// For example, for `(a)` it iterates only over `a`, skkipping segment heads `(` and `)`.
    pub fn iter_pat_match_subcrumbs<'a>(&'a self) -> impl Iterator<Item=AmbiguousCrumb> + 'a {
        self.iter_subcrumbs().filter(|crumb| {
            crumb.field != crate::crumbs::AmbiguousSegmentCrumb::Head
        })
    }
}
