'use strict'

import { app, BrowserWindow } from 'electron'
import * as path from 'path'
import { format as formatUrl } from 'url'

const is_development = process.env.NODE_ENV !== 'production'

let main_window_keep_alive

function create_main_window() {
    const window = new BrowserWindow(
        {
            width          : 1024,
            height         : 768,
            frame          : false,
        }
    )

    var search = "";
    if (app.commandLine.hasSwitch("debug")) {
        var value = app.commandLine.getSwitchValue("debug");
        search = "?debug";
        if (value !== undefined) {
            search += "=" + value
        }
    }

    if (is_development) {
        window.webContents.openDevTools()
            var url = `http://localhost:${process.env.ELECTRON_WEBPACK_WDS_PORT}/main/` + search;
            window.loadURL(url)
    } else {
        window.loadURL(formatUrl({
            pathname : path.join(__dirname.replace(/app\.asar$/, 'static'), '/main/index.html'),
            protocol : 'file',
            slashes  : true
        }) + search)
    }

    window.on('closed', () => {
        main_window_keep_alive = null
    })

    window.webContents.on('devtools-opened', () => {
        window.focus()
        setImmediate(() => {
            window.focus()
        })
    })

  return window
}

// quit application when all windows are closed
app.on('window-all-closed', () => {
    // on macOS it is common for applications to stay open until the user explicitly quits
    if (process.platform !== 'darwin') {
        app.quit()
    }
})

app.on('activate', () => {
    // on macOS it is common to re-create a window even after all windows have been closed
    if (main_window_keep_alive === null) {
        main_window_keep_alive = create_main_window()
    }
})

// create main BrowserWindow when electron is ready
app.on('ready', () => {
    main_window_keep_alive = create_main_window()
})
