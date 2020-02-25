//! This module provides NotificationService for displaying Notification messages.

use crate::prelude::*;

use wasm_bindgen::JsValue;



// ===================
// === JS Bindings ===
// ===================

#[wasm_bindgen(module = "/js/notification.js")]
extern "C" {
    #[allow(unsafe_code)]
    fn create_notification_service() -> JsValue;

    #[allow(unsafe_code)]
    fn create_notification
    (notification_service:&JsValue, content:JsValue, duration:JsValue, fade_out:JsValue);
}



// ===========================
// === NotificationService ===
// ===========================

/// NotificationService is responsible for displaying Notification messages.
#[derive(Debug,Clone)]
pub struct NotificationService {
    logger  : Logger,
    service : JsValue
}

impl NotificationService {
    /// Creates a new NotificationService.
    pub fn new(logger:&Logger) -> Self {
        let logger  = logger.sub("NotificationService");
        let service = create_notification_service();
        Self {logger,service}
    }

    /// Display an informational notification.
    pub fn info(&self, content:&str, duration:f64, fade_out:f64) {
        let msg = format!("Content: {}, duration: {}, transition: {}",content,duration,fade_out);
        let msg:&str = &msg;
        self.logger.info(msg);
        create_notification(&self.service,content.into(),duration.into(),fade_out.into());
    }

    /// Display a warning notification.
    pub fn warning(&self, content:&str, duration:f64, fade_out:f64) {
        let msg = format!("Content: {}, duration: {}, transition: {}",content,duration,fade_out);
        let msg:&str = &msg;
        self.logger.warning(msg);
        create_notification(&self.service,content.into(),duration.into(),fade_out.into());
    }

    /// Display an error notification.
    pub fn error(&self, content:&str, duration:f64, fade_out:f64) {
        let msg = format!("Content: {}, duration: {}, transition: {}",content,duration,fade_out);
        let msg:&str = &msg;
        self.logger.error(msg);
        create_notification(&self.service,content.into(),duration.into(),fade_out.into());
    }
}
