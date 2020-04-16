//! A module containing all actions provided by SpanTree.
//!
//! The actions are in WIP state - they will be implemented along connection operations.

use crate::Node;

/// An API for SpanTree nodes for doing actions.
#[allow(missing_docs)]
pub trait SpanTreeActions {
    fn can_set   (&self) -> bool;
    fn can_insert(&self) -> bool;
    fn can_erase (&self) -> bool;

    //TODO[ao] Add functions for actually do the action.
}

impl SpanTreeActions for Node {
    fn can_set   (&self) -> bool { false }
    fn can_insert(&self) -> bool { false }
    fn can_erase (&self) -> bool { false }
}
