//! The module with structures describing models shared between different controllers.

pub mod execution_context;
pub mod module;
pub mod synchronized;

pub use module::Module;
pub use execution_context::ExecutionContext;