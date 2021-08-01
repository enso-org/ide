//! Root module for compound geometries. Compound geometries are defined by using primitive
//! geometries and behave like smart constructors for commonly used shapes.

pub mod screen;
pub mod screen2;
pub mod sprite;



// ===============
// === Exports ===
// ===============

/// Common types.
pub mod types {
    use super::*;
    pub use screen::Screen;
    pub use screen2::Screen2;
    pub use sprite::{SpriteSystem,Sprite};
}
