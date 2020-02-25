'use strict'

import { app, BrowserWindow }  from 'electron'
import { format as formatUrl } from 'url'
import * as path               from 'path'

const is_development = process.env.NODE_ENV !== 'production'

let main_window_keep_alive

function create_main_window() {
    const window = new BrowserWindow({
        width  : 1024,
        height : 768,
        frame  : false,
    })

    let debug_scene = "";
    let command     = "debug";
    if (app.commandLine.hasSwitch(command)) {
        let value = app.commandLine.getSwitchValue(command);
        debug_scene = `?${command}`;
        if (value) {
            debug_scene += "=" + value
        }
    }

    if (is_development) {
        window.webContents.openDevTools();
        let url = `http://localhost:${process.env.ELECTRON_WEBPACK_WDS_PORT}/main/` + debug_scene;
        window.loadURL(url)
    } else {
        let static_path = __dirname.replace(/app\.asar$/, 'static');
        let url         = formatUrl({
            pathname : path.join(static_path, '/main/index.html'),
            protocol : 'file',
            slashes  : true
        }) + debug_scene;

        window.loadURL(url)
    }

    window.on('closed', () => {
        main_window_keep_alive = null
    })

  return window
}

app.on('window-all-closed', () => {
    // On macOS it is common for applications to stay open until the user explicitly quits.
    // Here we force the application to quit when all the windows are closed.
    if (process.platform !== 'darwin') {
        app.quit()
    }
})

app.on('activate', () => {
    // On macOS it is common to re-create a window even after all windows have been closed.
    if (main_window_keep_alive === null) {
        main_window_keep_alive = create_main_window()
    }
})

// Create main BrowserWindow when electron is ready.
app.on('ready', () => {
    main_window_keep_alive = create_main_window()
})
