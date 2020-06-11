//! Contains definition of trivial logger that discards all messages except warnings and errors.

use enso_prelude::*;

use crate::LogMsg;
use crate::AnyLogger;

use shapely::CloneRef;
use std::fmt::Debug;



// ==============
// === Logger ===
// ==============

/// Trivial logger that discards all messages except warnings and errors.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Logger {
    /// Path that is used as an unique identifier of this logger.
    pub path:ImString,
}



// ===================
// === Conversions ===
// ===================

impls!{ From + &From <crate::enabled::Logger> for Logger { |logger| Self::new(logger.path()) }}



// ======================
// === AnyLogger Impl ===
// ======================

impl AnyLogger for Logger {
    type This = Self;

    fn path(&self) -> &str {
        &self.path
    }

    fn new(path:impl Into<ImString>) -> Self {
        let path = path.into();
        Self {path}
    }

    fn trace       <Msg:LogMsg> (&self, _:Msg) {}
    fn debug       <Msg:LogMsg> (&self, _:Msg) {}
    fn info        <Msg:LogMsg> (&self, _:Msg) {}
    fn warning     <Msg:LogMsg> (&self, m:Msg) { crate::enabled::Logger::warning(&self.path,m) }
    fn error       <Msg:LogMsg> (&self, m:Msg) { crate::enabled::Logger::error  (&self.path,m) }
    fn group_begin <Msg:LogMsg> (&self, _:Msg) {}
    fn group_end                (&self       ) {}
}
