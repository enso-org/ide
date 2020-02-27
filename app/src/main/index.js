'use strict'

import { app, BrowserWindow }  from 'electron'
import { HttpServer}           from '../http-server'



// =================
// === Constants ===
// =================


// == Web Server ==

const HTTP_SERVER_PORT = 9090;


// === Execution mode ===

const MODE_PRODUCTION  = 'production';
const MODE_DEVELOPMENT = 'development';
const IS_DEVELOPMENT   = process.env.NODE_ENV === MODE_DEVELOPMENT


// === Window properties ===

const WINDOW_WIDTH      = 1024;
const WINDOW_HEIGHT     = 768;
const WINDOW_FRAME      = false;



// =======================
// === Server Creation ===
// =======================

var server = new HttpServer(`${__static}/main`, HTTP_SERVER_PORT);



// =======================
// === Window Creation ===
// =======================

let main_window_keep_alive

function create_main_window() {
    const window = new BrowserWindow({
        width  : WINDOW_WIDTH,
        height : WINDOW_HEIGHT,
        frame  : WINDOW_FRAME
    })

    let debug_scene = "";
    let command     = "debug";
    if (app.commandLine.hasSwitch(command)) {
        let value = app.commandLine.getSwitchValue(command);
        debug_scene = command;
        if (value) {
            debug_scene += `/${value}`;
        }
    }

    if (IS_DEVELOPMENT) {
        window.webContents.openDevTools();
    }
    window.loadURL(`http://localhost:${HTTP_SERVER_PORT}/${debug_scene}`);

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
    if (main_window_keep_alive === null) {
        main_window_keep_alive = create_main_window()
    }
})

// Create main BrowserWindow when electron is ready.
app.on('ready', () => {
    main_window_keep_alive = create_main_window()
})
