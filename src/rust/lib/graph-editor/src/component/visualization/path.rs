use crate::prelude::*;
use crate::data;



// ============
// === Name ===
// ============

im_string_newtype!{
    /// Name of the visualization. You cannot register two visualizations of the same name in the
    /// same library.
    Name
}



// ============
// === Path ===
// ============

#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct Path {
    pub library : data::LibraryName,
    pub name    : Name,
}

impl Path {
    pub fn new(library:impl Into<data::LibraryName>, name:impl Into<Name>) -> Self {
        let library = library.into();
        let name   = name.into();
        Self {library,name}
    }

    pub fn builtin(name:impl Into<Name>) -> Self {
        let library = data::builtin_library();
        Self::new(library,name)
    }
}
