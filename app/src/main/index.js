'use strict'

import { app, BrowserWindow }  from 'electron'
import * as HttpServer         from '../http-server'


// =================
// === Constants ===
// =================

const HTTP_SERVER_PORT = 9090

// === Execution mode ===

const MODE_PRODUCTION  = 'production'
const MODE_DEVELOPMENT = 'development'
const IS_DEVELOPMENT   = process.env.NODE_ENV === MODE_DEVELOPMENT


// === Window properties ===

const WINDOW_WIDTH      = 1024
const WINDOW_HEIGHT     = 768
const WINDOW_FRAME      = false


console.log("!!!",process.argv)
// =======================
// === Server Creation ===
// =======================
//console.log("!!!",__static)
var server = HttpServer.create({dir:'dist', port:HTTP_SERVER_PORT, fallback:'/assets/index.html'})


const { session } = require('electron')



// =======================
// === Window Creation ===
// =======================

let main_window_keep_alive

function create_main_window() {
    const window = new BrowserWindow({
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
        },
        width  : WINDOW_WIDTH,
        height : WINDOW_HEIGHT,
        frame  : WINDOW_FRAME
    })

    let debug_scene = ""
    let command     = "debug"
    if (app.commandLine.hasSwitch(command)) {
        let value = app.commandLine.getSwitchValue(command)
        debug_scene = command
        if (value) {
            debug_scene += `/${value}`
        }
    }

    if (IS_DEVELOPMENT) {
        window.webContents.openDevTools()
    }

//    session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
//            callback({
//                responseHeaders: {
//                    ...details.responseHeaders,
//                    'Content-Security-Policy': ["script-src 'self'; script-src-elem 'self' 'unsafe-inline'"]
//                }
//            })
//        })

    window.loadURL(`http://localhost:${HTTP_SERVER_PORT}/${debug_scene}`)

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



    if(!app.commandLine.hasSwitch("no-window")) {
        main_window_keep_alive = create_main_window()
    }
})
