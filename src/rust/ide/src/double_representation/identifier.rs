//! Module for types and utilities related to dealing with identifiers.

use crate::prelude::*;

use ast::crumbs::Located;


// ==================
// === Identifier ===
// ==================

// === Errors ===

/// Happens if a given string does not fulfill requirements of the referent name;
#[derive(Clone,Debug,Fail)]
#[fail(display="Operator `{}` cannot be made into var.",_0)]
pub struct OperatorCantBeMadeIntoVar(String);


// === Definition ===

/// Wrapper over an Ast that holds an atomic identifier of any kind.
///
/// Invariants: can get identifier name, the name is non-empty.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct Identifier(Ast);

impl Identifier {
    /// Wrap the `Ast` into `Identifier` if it actually is an identifier.
    pub fn new(ast:Ast) -> Option<Identifier> {
        let name = ast::identifier::name(&ast)?;
        (!name.is_empty()).as_some(Identifier(ast))
    }

    /// Get the identifier name.
    pub fn name(&self) -> &str {
        // Unwrap here is safe, as identifiers always allow obtaining an Identifier.
        ast::identifier::name(&self.0).unwrap()
    }

    /// Convert identifier to the variable form (i.e. non-referent). Fails if this is an operator.
    pub fn as_var(&self) -> Result<ast::Var,OperatorCantBeMadeIntoVar> {
        let name = self.name();
        // Unwrap below is safe, as identifier is always non-empty.
        let first_char = name.chars().next().unwrap();
        if first_char.is_alphabetic() {
            let name = name.to_lowercase();
            Ok(ast::Var {name})
        } else {
            Err(OperatorCantBeMadeIntoVar(name.to_owned()))
        }
    }
}


// === Implementations ===

impl PartialEq for Identifier {
    fn eq(&self, other:&Self) -> bool {
        self.name().eq(other.name())
    }
}

impl Eq for Identifier {}

impl Hash for Identifier {
    fn hash<H:std::hash::Hasher>(&self, state:&mut H) {
        self.name().hash(state)
    }
}



// ====================
// === ReferentName ===
// ====================

// === Errors ===

/// Happens if a given string does not fulfill requirements of the referent name;
#[derive(Clone,Debug,Fail)]
#[fail(display="The `{}` is not a valid referent name.",_0)]
pub struct NotReferentName(String);


// === Definition ===

#[derive(Clone,Debug,Display,Shrinkwrap,PartialEq,Eq,Hash)]
/// The name segment is a string that starts with an upper-cased character.
///
/// It is used for naming modules, module path segments and projects.
///
/// This value corresponds to contents of the `Cons` AST shape.
pub struct ReferentName(String);

impl ReferentName {
    /// Check if the given text would be a valid referent name;
    pub fn validate(name:impl AsRef<str>) -> Result<(), NotReferentName> {
        let name = name.as_ref();
        // TODO [mwu]
        //  We should be able to call parser or sth to verify that other requirements for the
        //  referent form identifiers are fulfilled.
        //  This is expected to become properly possible when the Rust rewrite of parser is done.
        //  See: https://github.com/enso-org/enso/issues/435
        let first_char = name.chars().next();
        match first_char {
            Some(c) if c.is_uppercase() => Ok(()),
            _                           => Err(NotReferentName(name.into())),
        }
    }

    /// Try interpreting given string as a referent name.
    ///
    /// Referent name is an identifier starting with an upper-cased letter, like `Maybe`.
    ///
    /// Fails if the given string is not a valid referent name (e.g. an empty string or lower-cased
    /// string).
    pub fn new(name:impl Str) -> Result<ReferentName, NotReferentName> {
        Self::validate(name.as_ref()).map(|_| ReferentName(name.into()))
    }
}


// === Implementations ===

impl AsRef<str> for ReferentName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}



// ======================
// === NormalizedName ===
// ======================

// === Definition ===

/// The identifier name normalized to a lower-case (as the comparisons are case-insensitive).
/// Implements case-insensitive compare with AST.
#[derive(Clone,Debug,Display,Hash,PartialEq,Eq)]
#[derive(Shrinkwrap)]
pub struct NormalizedName(String);

impl NormalizedName {
    /// Wraps given string into the normalized name.
    pub fn new(name:impl AsRef<str>) -> NormalizedName {
        let name = name.as_ref().to_lowercase();
        NormalizedName(name)
    }

    /// If the given AST is an identifier, returns its normalized name.
    pub fn try_from_ast(ast:&Ast) -> Option<NormalizedName> {
        ast::identifier::name(ast).map(NormalizedName::new)
    }

    /// Is the given string a prefix of this name.
    pub fn starts_with(&self, name:impl AsRef<str>) -> bool {
        let prefix = NormalizedName::new(name);
        self.0.starts_with(prefix.0.as_str())
    }
}


// === Implementations ===

/// Tests if Ast is identifier that might reference the same name (case insensitive match).
impl PartialEq<Ast> for NormalizedName {
    fn eq(&self, other:&Ast) -> bool {
        NormalizedName::try_from_ast(other).contains_if(|other_name| {
            other_name == self
        })
    }
}

impl From<NormalizedName> for String {
    fn from(name:NormalizedName) -> Self {
        name.0
    }
}

/// Case-insensitive identifier with its ast crumb location (relative to the node's ast).
pub type LocatedName = Located<NormalizedName>;
