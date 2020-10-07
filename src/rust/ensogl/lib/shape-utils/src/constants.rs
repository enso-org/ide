//! Useful constant for working with shapes.
//!
use ensogl_core::data::color;

// =================
// === Constants ===
// =================

/// Color to use if a shape should be invisible, but also should still receive mouse events.
pub const HOVER_COLOR : color::Rgba = color::Rgba::new(1.0,0.0,0.0,0.000_001);
