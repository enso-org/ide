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

    /// Display a new notification.
    pub fn notification(&self, content:&str, duration:f64, fade_out:f64) {
        let info = format!("Content: {}, duration: {}, transition: {}",content,duration,fade_out);
        let info:&str = &info;
        self.logger.info(info);
        create_notification(&self.service,content.into(),duration.into(),fade_out.into());
    }
}
