//! Contains definition of trivial logger that discards all messages.

use enso_prelude::*;

use crate::LogMsg;
use crate::AnyLogger;

use shapely::CloneRef;
use std::fmt::Debug;



// ==============
// === Logger ===
// ==============

/// Trivial logger that discards all the messages.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Logger{
    /// Path that is used as an unique identifier of this logger.
    pub path:Rc<String>,
}



// ===================
// === Conversions ===
// ===================

impls!{ From + &From <crate::enabled::Logger> for Logger { |logger| Self::new(logger.path()) }}



// ======================
// === AnyLogger Impl ===
// ======================

impl AnyLogger for Logger {
    fn path(&self) -> &str {
        self.path.as_str()
    }

    fn new(path:impl Str) -> Self {
        Self {path:Rc::new(path.into())}
    }

    fn trace      <M: LogMsg>(&self, _:M){}
    fn debug      <M: LogMsg>(&self, _:M){}
    fn info       <M: LogMsg>(&self, _:M){}
    fn warning    <M: LogMsg>(&self, _:M){}
    fn error      <M: LogMsg>(&self, _:M){}
    fn group_begin<M: LogMsg>(&self, _:M){}
    fn group_end             (&self)     {}
}
