'use strict'

import * as Electron   from 'electron'
import * as Server     from '../server'
import * as portfinder from 'portfinder'
import * as minimist   from 'minimist'




function kebabToCamelCase(str){
  let arr     = str.split('-');
  let capital = arr.map((item,index) => {
      return index ? item.charAt(0).toUpperCase() + item.slice(1).toLowerCase() : item
  })
  return capital.join("");
}

function parseCmdArgs() {
    let args = minimist(argv)
    for (let argName in args) {
        let newName = kebabToCamelCase(argName)
        args[newName] = args[argName]
    }
    return args
}



// =================
// === Constants ===
// =================

const APP_COMMAND           = "enso-studio"

let serverCfg = {
    port : 8080
}

let windowCfg = {
    width  : 1024,
    height : 768,
}

const HELP_MESSAGE = `
usage: ${APP_COMMAND} [options]

options:
    --debug-scene [SCENE]  Run the debug scene instead of the main app.
    --port                 Port to use [${serverCfg.port}].
    --help                 Print the help message and exit.
    --window-size [SIZE]   Set the window size [${windowCfg.width}x${windowCfg.height}].
    --version              Print the version and exit.
`

let argv = process.argv.slice(2)

// FIXME: https://github.com/electron-userland/electron-webpack/issues/354
if (argv[1] == '--') {
    argv.splice(1,1)
}





// ==================================
// === Command Line Args Handlers ===
// ==================================

let args = parseCmdArgs()

if (args.help) {
    console.log(HELP_MESSAGE)
    process.exit()
}

if (args.version) {
    // FIXME https://github.com/electron-userland/electron-webpack/issues/353
    console.log(Electron.app.getVersion());
    process.exit();
}

if (args.windowSize) {
    let size   = args.windowSize.split('x')
    let width  = parseInt(size[0])
    let height = parseInt(size[1])
    if (isNaN(width) || isNaN(height)) {
        console.error(`Incorrect window size provided '${args.windowSize}'.`)
    } else {
        windowCfg.width  = width
        windowCfg.height = height
    }
}

Object.assign(serverCfg,args)



// =================
// === Constants ===
// =================


// === Execution mode ===

const MODE_PRODUCTION  = 'production'
const MODE_DEVELOPMENT = 'development'
const IS_DEVELOPMENT   = process.env.NODE_ENV === MODE_DEVELOPMENT



// =======================
// === Server Creation ===
// =======================

serverCfg.dir      = 'dist'
serverCfg.fallback = '/assets/index.html'
var server = Server.create(serverCfg)


// ================
// === Security ===
// ================

// === WebView Security ===

/// A WebView created in a renderer process that does not have Node.js integration enabled will not
/// be able to enable integration itself. However, a WebView will always create an independent
/// renderer process with its own webPreferences. It is a good idea to control the creation of new
/// <webview> tags from the main process and to verify that their webPreferences do not disable
/// security features. Follow the link to learn more:
/// https://www.electronjs.org/docs/tutorial/security#11-verify-webview-options-before-creation
function secureWebPreferences(webPreferences) {
    if(!webPreferences) { webPreferences = {} }
    delete webPreferences.preload
    delete webPreferences.preloadURL
    delete webPreferences.nodeIntegration
    delete webPreferences.nodeIntegrationInWorker
    delete webPreferences.webSecurity
    delete webPreferences.allowRunningInsecureContent
    delete webPreferences.experimentalFeatures
    delete webPreferences.enableBlinkFeatures
    delete webPreferences.allowpopups
    webPreferences.contextIsolation = true
    return webPreferences
}

let urlWhitelist = []
Electron.app.on('web-contents-created', (event, contents) => {
    contents.on('will-attach-webview', (event, webPreferences, params) => {
        secureWebPreferences(webPreferences)
        if (!urlWhitelist.includes(params.src)) {
            event.preventDefault()
        }
    })
})


// === Prevent Navigation ===

/// Navigation is a common attack vector. If an attacker can convince your app to navigate away from
/// its current page, they can possibly force your app to open web sites on the Internet. Follow the
/// link to learn more:
/// https://www.electronjs.org/docs/tutorial/security#12-disable-or-limit-navigation
Electron.app.on('web-contents-created', (event,contents) => {
    contents.on('will-navigate', (event,navigationUrl) => {
        event.preventDefault()
        console.error(`Prevented navigation to '${navigationUrl}'`)
    })
})


// === Disable New Windows Creation ===

/// Much like navigation, the creation of new webContents is a common attack vector. Attackers
/// attempt to convince your app to create new windows, frames, or other renderer processes with
/// more privileges than they had before or with pages opened that they couldn't open before.
/// Follow the link to learn more:
/// https://www.electronjs.org/docs/tutorial/security#13-disable-or-limit-creation-of-new-windows
Electron.app.on('web-contents-created', (event,contents) => {
  contents.on('new-window', async (event,navigationUrl) => {
    event.preventDefault()
    console.error(`Blocking new window creation request to '${navigationUrl}'`)
  })
})


// =====================
// === Depreciations ===
// =====================

/// FIXME: Will not be needed in Electron 9 anymore.
Electron.app.allowRendererProcessReuse = true



// =======================
// === Window Creation ===
// =======================

let main_window_keep_alive

function createMainWindow() {
    const window = new Electron.BrowserWindow({
        webPreferences : secureWebPreferences(),
        width          : windowCfg.width,
        height         : windowCfg.height,
        frame          : false
    })




    if (IS_DEVELOPMENT) {
        window.webContents.openDevTools()
    }

    Electron.session.defaultSession.setPermissionRequestHandler (
        (webContents, permission, callback) => {
            const url = webContents.getURL()
            console.error(`Unhandled permission request '${permission}'.`)
            // https://www.electronjs.org/docs/tutorial/security#4-handle-session-permission-requests-from-remote-content
        }
    )

    let targetScene = ""
    if (args.debugScene) {
        targetScene = `debug/${args.debugScene}`
    }
    window.loadURL(`http://localhost:${serverCfg.port}/${targetScene}`)
//    window.loadURL(`chrome://flags/`)

    window.on('closed', () => {
        main_window_keep_alive = null
    })

  return window
}

Electron.app.on('window-all-closed', () => {
    // On macOS it is common for applications to stay open until the user explicitly quits.
    // Here we force the application to quit when all the windows are closed.
    if (process.platform !== 'darwin') {
        Electron.app.quit()
    }
})

Electron.app.on('activate', () => {
    if (main_window_keep_alive === null) {
        main_window_keep_alive = createMainWindow()
    }
})

Electron.app.commandLine.appendSwitch('disable-features', 'HardwareMediaKeyHandling,MediaSessionService')

// FIXME https://github.com/electron/electron/issues/22466
// FIXME https://github.com/electron/electron/issues/22465

// Create main BrowserWindow when electron is ready.
Electron.app.on('ready', () => {



    if(!Electron.app.commandLine.hasSwitch("no-window")) {
        main_window_keep_alive = createMainWindow()
    }
})



// TODO
// There are some errors like `DeprecationWarning: OutgoingMessage.prototype._headers is deprecated`
// Follow this topic to watch the resolution: https://github.com/http-party/http-server/issues/483
