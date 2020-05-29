//! Contains implementation of default logger.

use enso_prelude::*;

use crate::AnyLogger;
use crate::LogMsg;

use shapely::CloneRef;
use std::fmt::Debug;

#[cfg(target_arch = "wasm32")]
use web_sys::console;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;



// ==============
// === Logger ===
// ==============

/// Default Logger implementation.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Logger {
    /// Path that is used as an unique identifier of this logger.
    pub path:Rc<String>,
}

impl Logger {
    #[cfg(not(target_arch = "wasm32"))]
    fn format<M:LogMsg>(&self, msg:M) -> String {
        msg.with_log_msg(|s| format!("[{}] {}", self.path, s))
    }

    #[cfg(target_arch = "wasm32")]
    fn format<M:LogMsg>(&self, msg:M) -> JsValue {
        msg.with_log_msg(|s| format!("[{}] {}", self.path, s)).into()
    }
}



// ===================
// === Conversions ===
// ===================

impl From<crate::disabled::Logger> for Logger {
    fn from(logger:crate::disabled::Logger) -> Self {
        Self::new(logger.path())
    }
}



// ======================
// === AnyLogger Impl ===
// ======================

#[cfg(not(target_arch = "wasm32"))]
impl AnyLogger for Logger {
    fn path(&self) -> &str {
        self.path.as_str()
    }

    fn new(path:impl Str) -> Self {
        Self {path:Rc::new(path.into())}
    }

    fn trace<M:LogMsg>(&self, msg:M) {
        println!("{}",self.format(msg));
    }

    fn debug<M:LogMsg>(&self, msg:M) {
        println!("{}",self.format(msg));
    }

    fn info<M:LogMsg>(&self, msg:M) {
        println!("{}",self.format(msg));
    }

    fn warning<M:LogMsg>(&self, msg:M) {
        println!("[WARNING] {}",self.format(msg));
    }

    fn error<M:LogMsg>(&self, msg:M) {
        println!("[ERROR] {}",self.format(msg));
    }

    fn group_begin<M:LogMsg>(&self, msg:M) {
        println!(">>> {}",self.format(msg));
    }

    fn group_end(&self) {
        println!("<<<")
    }
}

#[cfg(target_arch = "wasm32")]
impl AnyLogger for Logger {
    fn path(&self) -> &str {
        self.path.as_str()
    }

    fn new(path:impl Str) -> Self {
        Self {path:Rc::new(path.into())}
    }

    fn trace<M:LogMsg>(&self, msg:M) {
        console::trace_1(&self.format(msg));
    }

    fn debug<M:LogMsg>(&self, msg:M) {
        console::debug_1(&self.format(msg));
    }

    fn info<M:LogMsg>(&self, msg:M) {
        console::info_1(&self.format(msg));
    }

    fn warning<M:LogMsg>(&self, msg:M) {
        console::warn_1(&self.format(msg));
    }

    fn error<M:LogMsg>(&self, msg:M) {
        console::error_1(&self.format(msg));
    }

    fn group_begin<M:LogMsg>(&self, msg:M) {
        console::group_1(&self.format(msg));
    }

    fn group_end(&self) {
        console::group_end();
    }
}
