//! This module contains all the submodules of the CSS3D rendering system.

mod html_object;
mod html_renderer;

pub use html_object::HtmlObject;
pub use html_renderer::HtmlRenderer;



// =============
// === Utils ===
// =============

/// eps is used to round very small values to 0.0 for numerical stability
pub fn eps(value: f32) -> f32 {
    if value.abs() < 1e-10 { 0.0 } else { value }
}
