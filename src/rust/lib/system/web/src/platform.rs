//! This module provides helpers for platform specific logic.

use super::window;

/// This enumeration lists all the supported platforms.
#[derive(Debug,Clone,Copy)]
pub enum Platform {
    Linux,
    Android,
    Windows,
    MacOS,
    IOS,
    Unknown
}

impl Platform {
    /// Queries which platform we are on.
    pub fn query() -> Self {
        let window = window();
        let navigator = window.navigator();
        let platform  = navigator.platform().unwrap_or_else(|_| "Unknown".into());
        let agent     = navigator.user_agent().unwrap_or_else(|_| "Unknown".into());

        const MACOS_PLATFORMS   : [&str; 4] = ["Macintosh", "MacIntel", "MacPPC", "Mac68K"];
        const WINDOWS_PLATFORMS : [&str; 4] = ["Win32", "Win64", "Windows", "WinCE"];
        const IOS_PLATFORMS     : [&str; 3] = ["iPhone", "iPad", "iPod"];

        if MACOS_PLATFORMS.iter().any(|item| **item == platform) {
            Platform::MacOS
        } else if IOS_PLATFORMS.iter().any(|item| **item == platform) {
            Platform::IOS
        } else if WINDOWS_PLATFORMS.iter().any(|item| **item == platform) {
            Platform::Windows
        } else if agent.find("Android").is_some() {
            Platform::Android
        } else if platform.find("Linux").is_some() {
            Platform::Linux
        } else {
            Platform::Unknown
        }
    }
}
