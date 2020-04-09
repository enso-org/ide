//! This module provides helpers for platform specific logic.

use super::window;

use enso_prelude::hashmap;

/// This enumeration lists all the supported platforms.
#[derive(Debug,Clone,Copy,PartialEq)]
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

        let platforms = hashmap!{
            String::from("Macintosh") => Platform::MacOS,
            String::from("MacIntel")  => Platform::MacOS,
            String::from("MacPPC")    => Platform::MacOS,
            String::from("Mac68K")    => Platform::MacOS,
            String::from("Win32")     => Platform::Windows,
            String::from("Win64")     => Platform::Windows,
            String::from("Windows")   => Platform::Windows,
            String::from("WinCE")     => Platform::Windows,
            String::from("iPhone")    => Platform::IOS,
            String::from("iPad")      => Platform::IOS,
            String::from("iPod")      => Platform::IOS
        };

        if let Some(platform) = platforms.get(&platform) {
            *platform
        } else if agent.find("Android").is_some() {
            Platform::Android
        } else if platform.find("Linux").is_some() {
            Platform::Linux
        } else {
            Platform::Unknown
        }
    }
}

#[cfg(all(test,any(host_os="linux",host_os="windows",host_os="macos")))]
mod test {
    use super::Platform;

    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    #[cfg(host_os = "linux")]
    fn host_platform() -> Platform {
        Platform::Linux
    }

    #[cfg(host_os = "windows")]
    fn host_platform() -> Platform {
        Platform::Windows
    }

    #[cfg(host_os = "macos")]
    fn host_platform() -> Platform {
        Platform::MacOS
    }

    #[wasm_bindgen_test]
    fn platform() {
        assert_eq!(Platform::query(), host_platform())
    }
}
