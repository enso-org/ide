//! Provides an API to send data to our remote logging service. Requires the remote logging
//! to be already set up on the JS side. That means, there needs to exist a `window.enso.remoteLog`
//! method that takes a string and does the actual logging.

use crate::data::*;

pub use wasm_bindgen::prelude::*;



#[wasm_bindgen(inline_js="
export function _remote_log(msg, value) {
    if (window !== undefined && window.enso !== undefined && window.enso.remoteLog !== undefined) {
        try {
            if (value === undefined) {
                window.enso.remoteLog(msg)
            } else {
                window.enso.remoteLog(msg,{value})
            }

        } catch (error) {
            console.error(\"Error while logging message. \" + error );
        }

    } else {
        console.warn(\"Failed to send log message.\")
    }
}

export function _remote_log_value(msg, field_name, value) {
    const data = {}
    data[field_name] = value
    _remote_log(msg,data)
}
")]
extern "C" {
    #[allow(unsafe_code)]
    fn _remote_log_value(msg:JsValue,field_name:JsValue,value:JsValue);
    #[allow(unsafe_code)]
    fn _remote_log(msg:JsValue,value:JsValue);
}

/// Send the provided public event to our logging service.
pub fn remote_log_event(message:&str) {
    _remote_log(JsValue::from(message.to_string()), JsValue::UNDEFINED);
}

/// Send the provided public event with a named value to our logging service.
pub fn remote_log_value
<T:Loggable>(message:&str, field_name:&str, data:AnonymousData<T>) {
    let msg = JsValue::from(message.to_string());
    let field_name = JsValue::from(field_name.to_string());
    _remote_log_value(msg, field_name, data.0.get());
}
