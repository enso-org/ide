use crate::prelude::*;



// ================
// === EnsoCode ===
// ================

/// Type alias for a string containing Enso code.
#[derive(Clone,Debug,Default,Eq,Hash,PartialEq)]
pub struct EnsoCode {
    content: ImString
}



// ================
// === EnsoType ===
// ================

/// Type alias for a string representing an Enso type.
#[derive(Clone,Debug,Default,Eq,Hash,PartialEq)]
pub struct EnsoType {
    content: ImString
}

impl EnsoType {
    pub fn new(content:impl Into<ImString>) -> Self {
        let content = content.into();
        Self {content}
    }

    pub fn any() -> Self {
        "Any".into()
    }
}

// TODO: all conversions in newtype macro

impl From<String> for EnsoType {
    fn from(content:String) -> Self {
        Self::new(content)
    }
}

impl From<&str> for EnsoType {
    fn from(content:&str) -> Self {
        Self::new(content)
    }
}
