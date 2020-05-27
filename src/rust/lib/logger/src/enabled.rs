use crate::*;
use enso_prelude::*;
use shapely::CloneRef;
use std::fmt::Debug;
use wasm_bindgen::JsValue;

#[cfg(target_arch = "wasm32")]
use web_sys::console;

/// Default Logger implementation.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Logger {
    pub path:Rc<String>,
}

#[allow(dead_code)]
impl Logger {
    fn format<M:LogMsg>(&self, msg:M) -> JsValue {
        msg.with_log_msg(|s| format!("[{}] {}", self.path, s)).into()
    }

    fn format2<M:LogMsg>(&self, msg:M) -> String {
        msg.with_log_msg(|s| format!("[{}] {}", self.path, s))
    }
}

impl LoggerApi for Logger {
    fn new<T:Str>(path:T) -> Self {
        let path = Rc::new(path.into());
        Self {path}
    }

    fn sub<T:Str>(&self, path:T) -> Self {
        if self.path.is_empty() { Self::new(path) } else {
            Self::new(format!("{}.{}", self.path, path.as_ref()))
        }
    }

    fn group<M:LogMsg,T,F:FnOnce() -> T>(&self, msg:M, f:F) -> T {
        self.group_begin(msg);
        let out = f();
        self.group_end();
        out
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn trace<M:LogMsg>(&self, msg:M) {
        println!("{}",self.format2(msg));
    }
    #[cfg(target_arch = "wasm32")]
    pub fn trace<M:LogMsg>(&self, _msg:M) {
        console::trace_1(&self.format(msg));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn debug<M:LogMsg>(&self, msg:M) {
        println!("{}",self.format2(msg));
    }
    #[cfg(target_arch = "wasm32")]
    pub fn debug<M:LogMsg>(&self, msg:M) {
        console::debug_1(&self.format(msg));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn info<M:LogMsg>(&self, msg:M) {
        println!("{}",self.format2(msg));
    }
    #[cfg(target_arch = "wasm32")]
    pub fn info<M:LogMsg>(&self, _msg:M) {
        console::info_1(&self.format(msg));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn warning<M:LogMsg>(&self, msg:M) {
        println!("[WARNING] {}",self.format2(msg));
    }
    #[cfg(target_arch = "wasm32")]
    pub fn warning<M:LogMsg>(&self, msg:M) {
        console::warn_1(&self.format(msg));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn error<M:LogMsg>(&self, msg:M) {
        println!("[ERROR] {}",self.format2(msg));
    }
    #[cfg(target_arch = "wasm32")]
    pub fn error<M:LogMsg>(&self, msg:M) {
        console::error_1(&self.format(msg));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn group_begin<M:LogMsg>(&self, msg:M) {
        println!(">>> {}",self.format2(msg));
    }
    #[cfg(target_arch = "wasm32")]
    pub fn group_begin<M:LogMsg>(&self, _msg:M) {
        console::group_1(&self.format(msg));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn group_end(&self) {
        println!("<<<")
    }    
    #[cfg(target_arch = "wasm32")]
    pub fn group_end(&self) {
        console::group_end();
    }
}
