//! GPU-specific types and related implementations.

pub mod data;
pub mod shader;


/// Common types.
pub mod types {
    use web_sys::WebGl2RenderingContext;

    pub use super::data::types::*;

    /// Alias for WebGl2RenderingContext.
    pub type Context = WebGl2RenderingContext;
}
