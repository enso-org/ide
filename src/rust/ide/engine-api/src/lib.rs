//! Crate containing the engine interface files.

#[allow(dead_code, unused_imports)]
use flatbuffers;

pub mod generated;

pub use generated::binary_protocol_generated as binary_protocol;
