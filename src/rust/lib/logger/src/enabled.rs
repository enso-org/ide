use crate::LogMsg;
use enso_prelude::*;
use shapely::CloneRef;
use std::fmt::Debug;
use wasm_bindgen::JsValue;

#[cfg(target_arch = "wasm32")]
use web_sys::console;

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
        let path = Rc::new(path.into());
        Self {path}
    }

    pub fn sub<T:Str>(&self, path: T) -> Self {
        if self.path.is_empty() {
            Self::new(path)
        } else {
            Self::new(format!("{}.{}", self.path, path.as_ref()))
        }
    }

    pub fn group<M: LogMsg, T, F: FnOnce() -> T>(&self, msg: M, f: F) -> T {
        self.group_begin(msg);
        let out = f();
        self.group_end();
        out
    }

    fn format<M: LogMsg>(&self, msg: M) -> JsValue {
        msg.with_log_msg(|s| format!("[{}] {}", self.path, s)).into()
    }

    fn format2<M: LogMsg>(&self, msg: M) -> String {
        msg.with_log_msg(|s| format!("[{}] {}", self.path, s))
    }
}

#[cfg(target_arch = "wasm32")]
impl Logger {
    pub fn trace<M: LogMsg>(&self, _msg: M) {
        //console::trace_1(&self.format(msg));
    }

    pub fn debug<M: LogMsg>(&self, msg: M) {
        console::debug_1(&self.format(msg));
    }

    pub fn info<M: LogMsg>(&self, _msg: M) {
        //console::info_1(&self.format(msg));
    }

    pub fn warning<M: LogMsg>(&self, msg: M) {
        console::warn_1(&self.format(msg));
    }

    pub fn error<M: LogMsg>(&self, msg: M) {
        console::error_1(&self.format(msg));
    }

    pub fn group_begin<M: LogMsg>(&self, _msg: M) {
        //console::group_1(&self.format(msg));
    }

    pub fn group_end(&self) {
        //console::group_end();
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Logger {
    pub fn trace<M: LogMsg>(&self, msg:M) {
        println!("{}",self.format2(msg));
    }
    pub fn debug<M: LogMsg>(&self, msg:M) {
        println!("{}",self.format2(msg));
    }
    pub fn info<M: LogMsg>(&self, msg: M) {
        println!("{}",self.format2(msg));
    }
    pub fn warning<M: LogMsg>(&self, msg: M) {
        println!("[WARNING] {}",self.format2(msg));
    }
    pub fn error<M: LogMsg>(&self, msg: M) {
        println!("[ERROR] {}",self.format2(msg));
    }
    pub fn group_begin<M: LogMsg>(&self, msg: M) {
        println!(">>> {}",self.format2(msg));
    }
    pub fn group_end(&self) {
        println!("<<<")
    }
}

