//! This is the root module for all geometry types, including polygon meshes, NURBS surfaces, and
//! volumes. It also contains compound geometry, predefined more complex shapes.

pub mod compound;
pub mod primitive;


// =================
// === Reexports ===
// =================

/// Common types.
pub mod types {
    use super::*;
    pub use primitive::types::*;
    pub use compound::types::*;
}
pub use types::*;
