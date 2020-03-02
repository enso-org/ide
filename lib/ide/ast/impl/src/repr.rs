use crate::*;

// ======================
// === Token Literals ===
// =====================

/// Token representing blank.
pub const BLANK_TOKEN:char = '_';

/// Symbol appearing after base of the number literal.
pub const NUMBER_BASE_SEPARATOR:char = '_';

/// Suffix to made a modifier from an operator
pub const MOD_SUFFIX:char = '=';

/// Symbol enclosing raw Text line.
pub const FMT_QUOTE:char = '\'';

/// Symbol enclosing formatted Text line.
pub const RAW_QUOTE:char = '"';

/// Symbol used to break lines in Text block.
pub const NEWLINE:char = '\n';

/// Symbol introducing escape segment in the Text.
pub const BACKSLASH:char = '\\';

/// Symbol enclosing expression segment in the formatted Text.
pub const EXPR_QUOTE:char = '`';

/// Symbol that introduces UTF-16 code in the formatted Text segment.
pub const UNICODE16_INTRODUCER:char = 'u';

/// String that opens "UTF-21" code in the formatted Text segment.
pub const UNICODE21_OPENER:&str = "u{";

/// String that closese "UTF-21" code in the formatted Text segment.
pub const UNICODE21_CLOSER:&str = "}";

/// Symbol that introduces UTF-16 code in the formatted Text segment.
pub const UNICODE32_INTRODUCER:char = 'U';

/// Quotes opening block of the raw text.
pub const RAW_BLOCK_QUOTES:&str = "\"\"\"";

/// Quotes opening block of the formatted text.
pub const FMT_BLOCK_QUOTES:&str = "'''";



// ===============
// === Builder ===
// ===============

tokenizer!(Empty);
tokenizer!(Letter, self.char);
tokenizer!(Space , self);
tokenizer!(Text  , self.str);
tokenizer!(Seq   , self.first, self.second);


// =====================
// === TextBlockLine ===
// =====================

/// Not an instance of `Tokenizer`, as it needs to know parent block's offset.
impl<T:Tokenizer> TextBlockLine<T> {
    fn tokenize(&self, builder:&mut impl TokenBuilder, offset:usize) {
        for empty_line_spaces in &self.empty_lines {
            (NEWLINE,empty_line_spaces).tokenize(builder);
        }
        (NEWLINE,offset,&self.text).tokenize(builder);
    }
}



// =====================
// === Text Segments ===
// =====================

tokenizer!(SegmentPlain    ,             self.value);
tokenizer!(SegmentRawEscape, BACKSLASH,  self.code );
tokenizer!(SegmentExpr<T>  , EXPR_QUOTE, self.value, EXPR_QUOTE);
tokenizer!(SegmentEscape   , BACKSLASH,  self.code );


// =================
// === RawEscape ===
// =================

tokenizer!(Unfinished);
tokenizer!(Invalid , self.str );
tokenizer!(Slash   , BACKSLASH);
tokenizer!(Quote   , FMT_QUOTE);
tokenizer!(RawQuote, RAW_QUOTE);


// ==============
// === Escape ===
// ==============

tokenizer!(EscapeCharacter , self.c     );
tokenizer!(EscapeControl   , self.name  );
tokenizer!(EscapeNumber    , self.digits);
tokenizer!(EscapeUnicode16 , UNICODE16_INTRODUCER, self.digits);
tokenizer!(EscapeUnicode21  ,UNICODE21_OPENER.deref() , self.digits
                             , UNICODE21_CLOSER.deref());
tokenizer!(EscapeUnicode32 , UNICODE32_INTRODUCER, self.digits);


// =============
// === Block ===
// =============

tokenizer!(BlockLine<T>, self.elem, self.off);


// =============
// === Macro ===
// =============

// === Macro Segments ==

tokenizer!(MacroMatchSegment<T> , self.head, self.body);
tokenizer!(MacroAmbiguousSegment, self.head, self.body);


// === MacroPatternMatch subtypes ===

tokenizer!(MacroPatternMatchRawBegin  );
tokenizer!(MacroPatternMatchRawEnd    );
tokenizer!(MacroPatternMatchRawNothing);
tokenizer!(MacroPatternMatchRawSeq    <T>, self.elem);
tokenizer!(MacroPatternMatchRawOr     <T>, self.elem);
tokenizer!(MacroPatternMatchRawMany   <T>, self.elem);
tokenizer!(MacroPatternMatchRawExcept <T>, self.elem);
tokenizer!(MacroPatternMatchRawBuild  <T>, self.elem);
tokenizer!(MacroPatternMatchRawErr    <T>, self.elem);
tokenizer!(MacroPatternMatchRawTag    <T>, self.elem);
tokenizer!(MacroPatternMatchRawCls    <T>, self.elem);
tokenizer!(MacroPatternMatchRawTok    <T>, self.elem);
tokenizer!(MacroPatternMatchRawBlank  <T>, self.elem);
tokenizer!(MacroPatternMatchRawVar    <T>, self.elem);
tokenizer!(MacroPatternMatchRawCons   <T>, self.elem);
tokenizer!(MacroPatternMatchRawOpr    <T>, self.elem);
tokenizer!(MacroPatternMatchRawMod    <T>, self.elem);
tokenizer!(MacroPatternMatchRawNum    <T>, self.elem);
tokenizer!(MacroPatternMatchRawText   <T>, self.elem);
tokenizer!(MacroPatternMatchRawBlock  <T>, self.elem);
tokenizer!(MacroPatternMatchRawMacro  <T>, self.elem);
tokenizer!(MacroPatternMatchRawInvalid<T>, self.elem);


// === Switch ===

tokenizer!(Switch<T>, self.get().deref());


// === Shifted ===

tokenizer!(Shifted    <T>, self.off,  self.wrapped);
tokenizer!(ShiftedVec1<T>, self.head, self.tail);


// =============================================================================
// === Shape ===================================================================
// =============================================================================

// ===============
// === Invalid ===
// ===============

tokenizer!(Unrecognized, self.str  );
tokenizer!(InvalidQuote, self.quote);
tokenizer!(InlineBlock , self.quote);


// ===================
// === Identifiers ===
// ===================

tokenizer!(Blank           , BLANK_TOKEN);
tokenizer!(Var             , self.name  );
tokenizer!(Cons            , self.name  );
tokenizer!(Opr             , self.name  );
tokenizer!(Mod             , self.name, MOD_SUFFIX );
tokenizer!(InvalidSuffix<T>, self.elem, self.suffix);


// ==============
// === Number ===
// ==============

/// Helper to represent that optional number base has additional character.
struct NumberBase<T>(T);

tokenizer!(NumberBase<T>, self.0, NUMBER_BASE_SEPARATOR);
tokenizer!(Number       , self.base.as_ref().map(NumberBase) , self.int);
tokenizer!(DanglingBase , self.base, NUMBER_BASE_SEPARATOR);



// ============
// === Text ===
// ============

// === Indented ===

/// Helper to represent line with additional spacing prepended.
struct Indented<T>(usize,T);

tokenizer!(Indented<T>, self.0, self.1);

impl<T> Block<T> {
    fn indented<'t, U>(&self, t:&'t U) -> Indented<&'t U> {
        Indented(self.indent,t)
    }
}


// === Lines ===

tokenizer!(TextLineRaw    , RAW_QUOTE, self.text, RAW_QUOTE);
tokenizer!(TextLineFmt<T> , FMT_QUOTE, self.text, FMT_QUOTE);


// === TextBlockRaw ==

impl Tokenizer for TextBlockRaw {
    fn tokenize(&self, builder:&mut impl TokenBuilder) {
        (RAW_BLOCK_QUOTES, self.spaces).tokenize(builder);
        for line in self.text.iter() {
            line.tokenize(builder, self.offset);
        }
    }
}


// === TextBlockFmt ==

impl<T:Tokenizer> Tokenizer for TextBlockFmt<T> {
    fn tokenize(&self, builder:&mut impl TokenBuilder) {
        (FMT_BLOCK_QUOTES,self.spaces).tokenize(builder);
        for line in self.text.iter() {
            line.tokenize(builder,self.offset);
        };
    }
}


// === TextUnclosed ==

// TODO: [mwu] `TextUnclosed<T>` as it needs to cut off closing quote from the
//             stored text line. Likely this type should be stored like this.

// TODO: [jv] this implementation is wrong since we cannot `pop` from TokenBuilder
//            either redesign TextUnclosed, so that it can use Tokenizer,
//            or come up with some smart/ugly hack
impl <T:Tokenizer> Tokenizer for TextUnclosed<T> {
    fn tokenize(&self, builder: &mut impl TokenBuilder) {
        self.line.tokenize(builder);
        // now we should remove missing quote
    }
}

// ====================
// === Applications ===
// ====================

tokenizer!(Infix<T>, self.larg, self.loff, self.opr, self.roff, self.rarg);

tokenizer!(Prefix      <T>, self.func, self.off, self.arg);
tokenizer!(SectionLeft <T>, self.arg,  self.off, self.opr);
tokenizer!(SectionRight<T>, self.opr,  self.off, self.arg);
tokenizer!(SectionSides<T>, self.opr);

// ==============
// === Module ===
// ==============

// === Module ==

impl<T:Tokenizer> Tokenizer for Module<T> {
    fn tokenize(&self, builder:&mut impl TokenBuilder) {
        let mut iter = self.lines.iter();
        if let Some(first_line) = iter.next() {
            first_line.tokenize(builder);
        }
        for line in iter {
            (NEWLINE,line).tokenize(builder);
        }
    }
}


// === Block ==

impl<T:Tokenizer> Tokenizer for Block<T> {
    fn tokenize(&self, builder:&mut impl TokenBuilder) {
        (!self.is_orphan).as_some(NEWLINE).tokenize(builder);
        for empty_line_space in &self.empty_lines {
            (empty_line_space,NEWLINE).tokenize(builder);
        }
        self.indented(&self.first_line).tokenize(builder);
        for line in &self.lines {
            (NEWLINE,self.indented(line)).tokenize(builder);
        }
    }
}



// ==============
// === Macros ===
// ==============

// === Match ==

impl<T:Tokenizer> Tokenizer for Match<T> {
    fn tokenize(&self, builder:&mut impl TokenBuilder) {
        for pat_match in &self.pfx {
            for sast in pat_match.iter() {
                // reverse the order for prefix: ast before spacing
                (&sast.wrapped,&sast.off).tokenize(builder);
            }
        }
        self.segs.tokenize(builder);
    }
}


// === Ambiguous ===

tokenizer!(Ambiguous, self.segs);


// =====================
// === Spaceless AST ===
// =====================

no_tokenizer!(Comment);
no_tokenizer!(Import<T>);
no_tokenizer!(Mixfix<T>);
no_tokenizer!(Group<T>);
no_tokenizer!(Def<T>);
no_tokenizer!(Foreign);



// =============
// === Tests ===
// =============

/// Tests for spacelesss AST. Other AST is covered by parsing tests that verify
/// that correct spans and text representation are generated. Only spaceless AST
/// is not returned by the parser and can't be covered in this way.
#[cfg(test)]
mod tests {
    use super::*;

    // === Comment ===

    fn make_comment() -> Shape<Ast> {
        Comment {lines:vec![]}.into()
    }

    #[test]
    #[should_panic]
    fn comment_panics_on_repr() {
        make_comment().repr();
    }

    #[test]
    #[should_panic]
    fn comment_panics_on_span() {
        make_comment().span();
    }


    // === Import ===

    fn make_import() -> Shape<Ast> {
        Import {path : vec![]}.into()
    }

    #[test]
    #[should_panic]
    fn import_panics_on_repr() {
        make_import().repr();
    }

    #[test]
    #[should_panic]
    fn import_panics_on_span() {
        make_import().span();
    }


    // === Mixfix ===

    fn make_mixfix() -> Shape<Ast> {
        Mixfix {
            name : vec![],
            args : vec![]
        }.into()
    }

    #[test]
    #[should_panic]
    fn mixfix_panics_on_repr() {
        make_mixfix().repr();
    }

    #[test]
    #[should_panic]
    fn mixfix_panics_on_span() {
        make_mixfix().span();
    }


    // === Group ===

    fn make_group() -> Shape<Ast> {
        Group {body : None}.into()
    }

    #[test]
    #[should_panic]
    fn group_panics_on_repr() {
        make_group().repr();
    }

    #[test]
    #[should_panic]
    fn group_panics_on_span() {
        make_group().span();
    }


    // === Def ===

    fn make_def() -> Shape<Ast> {
        Def {
            name : Ast::cons("Foo"),
            args : vec![],
            body : None
        }.into()
    }

    #[test]
    #[should_panic]
    fn def_panics_on_repr() {
        make_def().repr();
    }

    #[test]
    #[should_panic]
    fn def_panics_on_span() {
        make_def().span();
    }

    // === Foreign ===

    fn make_foreign() -> Shape<Ast> {
        Foreign {
            indent : 0,
            lang   : "Python".into(),
            code   : vec![]
        }.into()
    }

    #[test]
    #[should_panic]
    fn foreign_panics_on_repr() {
        make_foreign().repr();
    }

    #[test]
    #[should_panic]
    fn foreign_panics_on_span() {
        make_foreign().span();
    }
}
