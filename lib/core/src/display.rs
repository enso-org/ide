//! Root module for all display-related abstractions, including display objects, shapes, geometries,
//! rendering utilities, etc.

pub mod camera;
pub mod shape;
pub mod symbol;
pub mod object;
pub mod scene;
pub mod world;
pub mod navigation;



/// Commonly used types.
pub mod types {
    use super::*;
    pub use scene::Scene;
}
pub use types::*;
