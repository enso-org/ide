use crate::*;
use enso_prelude::*;
use shapely::CloneRef;
use std::fmt::Debug;



/// Trivial logger that discards all the messages.
#[derive(Clone,Copy,CloneRef,Debug,Default)]
pub struct Logger();

impl From<enabled::Logger> for Logger {
    fn from(_:enabled::Logger) -> Self { Logger() }
}

impl From<&enabled::Logger> for Logger {
    fn from(_:&enabled::Logger) -> Self { Logger() }
}

impl LoggerApi for Logger {
    fn new<T: Str>(_:T) -> Self {
        Logger()
    }

    fn sub<T:Str>(&self, _:T) -> Logger {
        Logger()
    }

    fn group<M: LogMsg,T,F:FnOnce() -> T>(&self, _:M, f:F) -> T {
        f()
    }

    fn trace      <M: LogMsg>(&self, _:M){}
    fn debug      <M: LogMsg>(&self, _:M){}
    fn info       <M: LogMsg>(&self, _:M){}
    fn warning    <M: LogMsg>(&self, _:M){}
    fn error      <M: LogMsg>(&self, _:M){}
    fn group_begin<M: LogMsg>(&self, _:M){}
    fn group_end             (&self)     {}
}
