//! This module implements GPU-based texture support. Proper texture handling is a complex topic.
//! Follow the link to learn more about many assumptions this module was built upon:
//! https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D

pub mod types;
pub mod storage;
pub mod class;

pub use class::*;
pub use types::*;
pub use storage::*;
