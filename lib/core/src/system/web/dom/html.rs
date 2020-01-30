//! This module contains all the submodules of the CSS3D rendering system.

mod css3d_system;
mod css3d_object;
mod css3d_renderer;

pub use css3d_object::Css3dObject;
pub use css3d_object::Css3dPosition;
pub use css3d_renderer::Css3dRenderer;
pub use css3d_system::Css3dSystem;



// =============
// === Utils ===
// =============

/// eps is used to round very small values to 0.0 for numerical stability
pub fn eps(value: f32) -> f32 {
    if value.abs() < 1e-10 { 0.0 } else { value }
}
