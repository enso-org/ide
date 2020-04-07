use crate::prelude::*;

// ==========
// === Id ===
// ==========

/// Identifier of FRP node. Used mainly for debug purposes.
#[derive(Debug,Clone,CloneRef,Copy,Eq,From,Hash,Into,PartialEq)]
pub struct Id {
    raw : usize
}

/// Implementors of this trait has to be assigned with an unique Id. All FRP nodes implement it.
#[allow(missing_docs)]
pub trait HasId {
    fn id(&self) -> Id;
}



// =============
// === Label ===
// =============

/// FRP node label. USed mainly for debugging purposes.
pub type Label = &'static str;

/// Implementors of this trait has to be assigned with a label. Each FRP node implements it.
#[allow(missing_docs)]
pub trait HasLabel {
    fn label(&self) -> Label;
}