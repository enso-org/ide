//! Provides an API to send data to our remote logging service. Requires the remote logging
//! to be already set up on the JS side. That means, there needs to exist a `window.enso.remote_log`
//! method that takes a string and does the actual logging.

use js_sys;
use wasm_bindgen::JsValue;

/// Send the provided public event to our logging service.
pub fn remote_log(message:crate::data::Public) {
    let js_new  = js_sys::Function::new_with_args("msg", "window.enso.remote_log(msg)");
    let context = JsValue::NULL;
    let _res = js_new.call1(&context,&message.data.into());
}

/// Send the provided public event with data to our logging service.
pub fn remote_log_data(message:crate::data::Public,data:crate::data::Public) {
    let js_new  = js_sys::Function::new_with_args("msg,data", "window.enso.remote_log(msg,{data:data})");
    let context = JsValue::NULL;
    let _res = js_new.call2(&context,&message.data.into(),&data.data.into());
}

