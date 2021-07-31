//! Utilities for dealing with macro-related parts of AST and language, including `Match` shape and
//! such constructs as lambda expressions.


use crate::prelude::*;

use crate::crumbs::AmbiguousCrumb;
use crate::crumbs::Located;
use crate::crumbs::MatchCrumb;
use crate::known;
use crate::Shifted;



// ==================================
// === Recognized Macros Keywords ===
// ==================================

/// The keyword introducing a disabled code line.
pub const DISABLING_COMMENT_INTRODUCER:&str = "#";

/// The keyword introducing a documentation block.
pub const DOCUMENTATION_COMMENT_INTRODUCER:&str = "##";

/// The keyword introducing an qualified import declaration. See:
/// https://dev.enso.org/docs/enso/syntax/imports.html#import-syntax
pub const QUALIFIED_IMPORT_KEYWORD:&str = "import";

/// The keyword introducing an unqualified import declaration.
pub const UNQUALIFIED_IMPORT_KEYWORD:&str = "from";

/// The keyword introducing an unqualified export declaration.
pub const QUALIFIED_EXPORT_KEYWORD:&str = "export";



// ================
// === Comments ===
// ================

// === Disable Comments ===

/// Try Interpreting the line as disabling comment. Return the text after `#`.
pub fn as_disable_comment(ast:&Ast) -> Option<String> {
    let r#match       = crate::known::Match::try_from(ast).ok()?;
    let first_segment = &r#match.segs.head;
    if crate::identifier::name(&first_segment.head) == Some(DISABLING_COMMENT_INTRODUCER) {
        Some(first_segment.body.repr())
    } else {
        None
    }
}

/// Check if this AST is a disabling comment.
pub fn is_disable_comment(ast:&Ast) -> bool {
    as_disable_comment(ast).is_some()
}


// === Documentation Comments ===

/// Ast known to be a documentation comment.
#[derive(Clone,Debug)]
pub struct DocCommentInfo {
    ast              : known::Match,
    body             : crate::MacroPatternMatch<Shifted<Ast>>,
    /// The absolute indent of the block that contains the line with documentation comment.
    pub block_indent : usize,
}

impl DocCommentInfo {
    /// Try constructing from AST, return None if this is not a documentation comment.
    pub fn new(ast:&Ast) -> Option<Self> {
        Self::new_indented(ast,0)
    }

    /// Creates a documentation from Ast and information about indentation of the block it belongs
    /// to.
    pub fn new_indented(ast:&Ast, block_indent:usize) -> Option<Self> {
        let ast                = crate::known::Match::try_from(ast).ok()?;
        let first_segment      = &ast.segs.head;
        let introducer         = crate::identifier::name(&first_segment.head)?;
        let introducer_matches = introducer == DOCUMENTATION_COMMENT_INTRODUCER;
        let body               = first_segment.body.clone_ref();
        introducer_matches.then(|| DocCommentInfo {ast,body,block_indent})
    }

    /// Get the documentation comment's AST.
    pub fn ast(&self) -> known::Match {
        self.ast.clone_ref()
    }

    /// Get the documentation text.
    pub fn text(&self) -> String {
        // This gets us documentation text, however non-first lines have the absolute indent
        // whitespace preserved. Thus, we remove spurious indent, to keep only the relative indent
        // to the comment's inner block (i.e. the right after the `##` introducer).
        let repr   = self.body.repr();
        let indent = self.block_indent + DOCUMENTATION_COMMENT_INTRODUCER.len();
        let old    = format!("\n{}", " ".repeat(indent));
        let new    = "\n";
        repr.replace(&old,new)
    }

    /// Get the documentation text.
    pub fn text_to_repr(text:&str) -> String {
        let mut lines     = text.lines();
        let first_line    = lines.next().map(|line| iformat!("##{line}"));
        let other_lines   = lines       .map(|line| iformat!("  {line}"));
        let mut out_lines = first_line.into_iter().chain(other_lines);
        out_lines.join("\n")
    }
}

impl AsRef<Ast> for DocCommentInfo {
    fn as_ref(&self) -> &Ast {
        self.ast.ast()
    }
}

impl Display for DocCommentInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.text())
    }
}

/// Check if given Ast stores a documentation comment.
pub fn is_documentation_comment(ast:&Ast) -> bool {
    DocCommentInfo::new(ast).is_some()
}



// ===============
// === Imports ===
// ===============

/// If the given AST node is an import declaration, returns it as a Match (which is the only shape
/// capable of storing import declarations). Returns `None` otherwise.
pub fn ast_as_import_match(ast:&Ast) -> Option<known::Match> {
    let macro_match = known::Match::try_from(ast).ok()?;
    is_match_import(&macro_match).then_some(macro_match)
}

/// Check if the given macro match node is an import declaration.
pub fn is_match_import(ast:&known::Match) -> bool {
    let segment = &ast.segs.head;
    let keyword = crate::identifier::name(&segment.head);
    if keyword.contains_if(|str| *str == UNQUALIFIED_IMPORT_KEYWORD) {
        let second_segment = &ast.segs.tail.first();
        match second_segment {
            Some(seg) => {
                let keyword_2 = crate::identifier::name(&seg.head);
                if keyword_2.contains_if(|str| *str == QUALIFIED_IMPORT_KEYWORD) {
                    return true
                }
            }
            None => return false
        }
    }
    keyword.contains_if(|str| *str == QUALIFIED_IMPORT_KEYWORD)
}

/// Check if the given ast node is an import declaration.
pub fn is_ast_import(ast:&Ast) -> bool {
    ast_as_import_match(ast).is_some()
}



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
    crate::opr::is_arrow_opr(&segment.head).then_some(macro_match)
}

/// Describes the given Ast as lambda, if this is a matched `->` builtin macro.
pub fn as_lambda(ast:&Ast) -> Option<LambdaInfo> {
    let _              = as_lambda_match(ast)?;
    let mut child_iter = ast.iter_subcrumbs();
    let arg            = ast.get_located(child_iter.next()?).ok()?;
    let opr            = ast.get_located(child_iter.next()?).ok()?;
    let body           = ast.get_located(child_iter.next()?).ok()?;
    let is_arrow       = crate::opr::is_arrow_opr(opr.item);
    is_arrow.then_some(LambdaInfo {arg,opr,body})
}



// ===================
// === Match Utils ===
// ===================

impl crate::Match<Ast> {
    /// Iterates matched ASTs. Skips segment heads ("keywords").
    /// For example, for `(a)` it iterates only over `a`, skkipping segment heads `(` and `)`.
    pub fn iter_pat_match_subcrumbs(&self) -> impl Iterator<Item=MatchCrumb> + '_ {
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
    pub fn iter_pat_match_subcrumbs(&self) -> impl Iterator<Item=AmbiguousCrumb> + '_ {
        self.iter_subcrumbs().filter(|crumb| {
            crumb.field != crate::crumbs::AmbiguousSegmentCrumb::Head
        })
    }
}
