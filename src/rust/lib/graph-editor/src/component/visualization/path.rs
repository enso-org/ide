//! Definition of name and path which define unique visualization definition location.

use crate::prelude::*;
use crate::data;



// ============
// === Name ===
// ============

im_string_newtype!{
    /// Name of the visualization. You cannot define two visualizations of the same name in the
    /// same library.
    Name
}



// ============
// === Path ===
// ============

/// A fully qualified path of a visualization definition. Contains both the library name and the
/// visualization name.
#[derive(Clone,CloneRef,Debug,Eq,Hash,PartialEq)]
#[allow(missing_docs)]
pub struct Path {
    pub library : data::LibraryName,
    pub name    : Name,
}

impl Path {
    /// Constructor.
    pub fn new(library:impl Into<data::LibraryName>, name:impl Into<Name>) -> Self {
        let library = library.into();
        let name   = name.into();
        Self {library,name}
    }

    /// Constructor for builtin visualizations.
    pub fn builtin(name:impl Into<Name>) -> Self {
        let library = data::builtin_library();
        Self::new(library,name)
    }
}
