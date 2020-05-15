//! This module contains synchronising wrappers for models whose state are a reflection of
//! Language Server state, e.g. modules, execution contexts etc. These wrappers synchronize both
//! states by notifying Language Server of every change and listening on LanguageServer.
pub mod execution_context;
#[allow(missing_docs)] //TODO[ao]
pub mod module;

pub use execution_context::ExecutionContext;
