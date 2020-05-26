use crate::LogMsg;
use enso_prelude::*;
use shapely::CloneRef;
use std::fmt::Debug;



// ==============
// === Logger ===
// ==============

#[derive(Clone,CloneRef,Debug,Default)]
pub struct Logger {
    pub path: Rc<String>,
}

#[allow(dead_code)]
impl Logger {
    pub fn new<T:Str>(path:T) -> Self {
        Self {path:Rc::new(path.into())}
    }

    pub fn sub<T:Str>(&self, path:T) -> Self {
        if self.path.is_empty() {
            Self::new(path)
        } else {
            Self::new(format!("{}.{}", self.path, path.as_ref()))
        }
    }

    pub fn group<M:LogMsg, T, F:FnOnce() -> T>(&self, _msg:M, f:F) -> T {
        f()
    }
}

impl Logger {
    pub fn trace      <M:LogMsg>(&self, _msg:M) {}
    pub fn debug      <M:LogMsg>(&self, _msg:M) {}
    pub fn info       <M:LogMsg>(&self, _msg:M) {}
    pub fn warning    <M:LogMsg>(&self, _msg:M) {}
    pub fn error      <M:LogMsg>(&self, _msg:M) {}
    pub fn group_begin<M:LogMsg>(&self, _msg:M) {}
    pub fn group_end            (&self        ) {}
}