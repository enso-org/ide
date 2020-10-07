//! ListView EnsoGL Component.
//!
//! ListView a displayed list of entries with possibility of selecting one and "chosing" by
//! clicking or pressing enter - similar to the HTML `<select>`.

pub mod component;
pub mod entry;

/// Commonly used types and functions.
pub mod prelude {
    pub use ensogl_core::prelude::*;
}

pub use component::ListView;
