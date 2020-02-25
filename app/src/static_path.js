
/// Check if the code is running in Electron.
function is_electron() {
    // Renderer process
    if (typeof window !== 'undefined' && typeof window.process === 'object' && window.process.type === 'renderer') {
        return true;
    }

    // Main process
    if (typeof process !== 'undefined' && typeof process.versions === 'object' && !!process.versions.electron) {
        return true;
    }

    // Detect the user agent when the `nodeIntegration` option is set to true
    if (typeof navigator === 'object' && typeof navigator.userAgent === 'string' && navigator.userAgent.indexOf('Electron') >= 0) {
        return true;
    }

    return false;
}

/// Check if the code is running in development mode.
function is_development() {
    if (is_electron()) {
        return typeof process       !=  'undefined'
            && typeof process.env   === 'object'
            && process.env.NODE_ENV ==  'development';
    } else {
        return true;
    }
}

/// Get static path.
export function get_static_path(dirname) {
//    if (is_electron() && !is_development()) {
        return dirname.replace(/app\.asar$/, 'static');
//    } else {
//        return "./"
//    }
}
