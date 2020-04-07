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
        let platform  = navigator.platform().unwrap_or("Unknown".into());
        let agent     = navigator.user_agent().unwrap_or("Unknown".into());

        const MACOS_PLATFORMS   : [&str; 4] = ["Macintosh", "MacIntel", "MacPPC", "Mac68K"];
        const WINDOWS_PLATFORMS : [&str; 4] = ["Win32", "Win64", "Windows", "WinCE"];
        const IOS_PLATFORMS     : [&str; 3] = ["iPhone", "iPad", "iPod"];

        if let Some(_) = MACOS_PLATFORMS.iter().find(|item| **item == platform) {
            Platform::MacOS
        } else if let Some(_) = IOS_PLATFORMS.iter().find(|item| **item == platform) {
            Platform::IOS
        } else if let Some(_) = WINDOWS_PLATFORMS.iter().find(|item| **item == platform) {
            Platform::Windows
        } else if let Some(_) = agent.find("Android") {
            Platform::Android
        } else if let Some(_) = platform.find("Linux") {
            Platform::Linux
        } else {
            Platform::Unknown
        }
    }
}
