'use strict'

import cfg from '../../../config'
import * as assert    from 'assert'
import * as buildCfg  from '../../../../../dist/build.json'
import * as Electron  from 'electron'
import * as isDev     from 'electron-is-dev'
import * as minimist  from 'minimist'
import * as path      from 'path'
import * as pkg       from '../package.json'
import * as rootCfg   from '../../../package.json'
import * as Server    from 'enso-studio-common/src/server'
import * as util      from 'util'
import * as yargs     from 'yargs'

import paths from '../../../../../build/paths'

const child_process = require('child_process')
const fss = require('fs')



// =============
// === Paths ===
// =============

const root = Electron.app.getAppPath()
const resources = path.join(root, "..")



// FIXME default options parsed wrong
// https://github.com/yargs/yargs/issues/1590

// ================
// === Defaults ===
// ================

let windowCfg = {
    width  : 640,
    height : 640,
}



// =============
// === Utils ===
// =============

function capitalizeFirstLetter(string) {
  return string.charAt(0).toUpperCase() + string.slice(1)
}

const execFile = util.promisify(child_process.execFile);



// =====================
// === Option Parser ===
// =====================

let usage = `
${pkg.build.productName} ${rootCfg.version} command line interface.

Usage: ${pkg.build.productName} [options]
`

let optParser = yargs
    .scriptName("")
    .usage(usage)
    .help()
    .version(false)
    .parserConfiguration({'populate--':true})
    .strict()


// === Config Options ===

let configOptionsGroup = 'Config Options:'

optParser.options('port', {
    group    : configOptionsGroup,
    describe : `Port to use [${Server.DEFAULT_PORT}]`,
})

optParser.options('project', {
    group    : configOptionsGroup,
    describe : 'Open the specified project on startup',
})

optParser.options('server', {
    group    : configOptionsGroup,
    describe : 'Run the server [true]',
})

optParser.options('window', {
    group    : configOptionsGroup,
    describe : 'Show the window [true]',
})

optParser.options('background-throttling', {
    group    : configOptionsGroup,
    describe : 'Throttle animations when run in background [false]',
})

optParser.options('backend', {
    group    : configOptionsGroup,
    describe : 'Start the backend process automatically [true]',
})

optParser.options('backend-path', {
    group    : configOptionsGroup,
    describe : 'Set the path of a local project manager to use for running projects',
})

// === Debug Options ===

let debugOptionsGroup = 'Debug Options:'

optParser.options('entry-point', {
    group       : debugOptionsGroup,
    describe    : 'Run an alternative entry point (e.g. one of the debug scenes)',
//    requiresArg : true
})

optParser.options('dev', {
    group       : debugOptionsGroup,
    describe    : 'Run the application in development mode',
})

optParser.options('devtron', {
    group       : debugOptionsGroup,
    describe    : 'Install the Devtron Developer Tools extension',
})


// === Style Options ===

let styleOptionsGroup = 'Style Options:'

optParser.options('frame', {
    group       : styleOptionsGroup,
    describe    : 'Draw window frame. Defaults to `false` on MacOS and `true` otherwise.',
    type        : `boolean`
})

optParser.options('vibrancy', {
    group       : styleOptionsGroup,
    describe    : 'Use the vibrancy effect',
    default     : false,
    type        : `boolean`
})

optParser.options('window-size', {
    group       : styleOptionsGroup,
    describe    : `Set the window size [${windowCfg.width}x${windowCfg.height}]`,
    requiresArg : true
})


// === Other Options ===

optParser.options('info', {
    describe    : `Print the system debug info`,
})

optParser.options('version', {
    describe    : `Print the version`,
})

optParser.options('crash-report-host', {
    describe    : 'The address of the server that will receive crash reports. ' +
                  'Consists of a hostname, optionally followed by a ":" and a port number',
    requiresArg : true,
    default: cfg.defaultLogServerHost
})


// === Parsing ===

function parseCmdArgs() {
    let argv = isDev ? process.argv.slice(process.argv.indexOf('--') + 1) : process.argv
    return optParser.parse(argv)
}

let args = parseCmdArgs()

// Note: this is a conditional default to avoid issues with some window managers affecting
// interactions at the top of a borderless window. Thus, we want borders on Win/Linux and
// borderless on Mac. See https://github.com/enso-org/ide/issues/1101 and
// https://github.com/electron/electron/issues/3647 for details.
if (args.frame === undefined) {
    args.frame = (process.platform !== 'darwin')
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



// ==================
// === Debug Info ===
// ==================

let versionInfo = {
    version: rootCfg.version,
    build: buildCfg.buildVersion,
    electron: process.versions.electron,
    chrome: process.versions.chrome,
}

async function getDebugInfo() {
    let procMemInfo = await process.getProcessMemoryInfo()
    return {
        version: versionInfo,
        creation: process.getCreationTime(),
        perf: {
            cpu: process.getCPUUsage(),
        },
        memory: {
            heap: process.getHeapStatistics(),
            blink: process.getBlinkMemoryInfo(),
            process: procMemInfo,
            system: process.getSystemMemoryInfo(),
        },
        system: {
            platform: process.platform,
            arch: process.arch,
            version: process.getSystemVersion(),
        },
    }
}

async function printDebugInfo() {
    let info = await getDebugInfo()
    console.log(JSON.stringify(info,undefined,4))
    process.exit();
}



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
    // TODO[WD]: We may want to enable it and use IPC to communicate with preload script.
    //           https://stackoverflow.com/questions/38335004/how-to-pass-parameters-from-main-process-to-render-processes-in-electron
    // webPreferences.contextIsolation = true
    webPreferences.enableRemoteModule = true
    return webPreferences
}

let urlWhitelist = []
Electron.app.on('web-contents-created', (event,contents) => {
    contents.on('will-attach-webview', (event,webPreferences,params) => {
        secureWebPreferences(webPreferences)
        if (!urlWhitelist.includes(params.src)) {
            console.error(`Blocked the creation of WebView pointing to '${params.src}'`)
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
        const parsedUrl = new URL(navigationUrl)
        if (parsedUrl.origin !== origin) {
            event.preventDefault()
            console.error(`Prevented navigation to '${navigationUrl}'.`)
        }
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



// =======================
// === Project Manager ===
// =======================

async function withBackend(opts) {
    let binPath = args['backend-path']
    if (!binPath) {
        binPath = paths.get_project_manager_path(resources)
    }
    let binExists = fss.existsSync(binPath)
    assert(binExists, `Could not find the project manager binary at ${binPath}.`)

    let out = await execFile(binPath,opts).catch(function(err) {throw err})
    return out
}

function runBackend() {
    if(args.backend !== false) {
        console.log("Starting the backend process.")
        withBackend()
    }
}

async function backendVersion() {
    if(args.backend !== false) {
        return await withBackend(['--version']).then((t) => t.stdout)
    }
}



// ============
// === Main ===
// ============

let hideInsteadOfQuit = false

let server     = null
let mainWindow = null

async function main() {
    runBackend()
    console.log("Starting the IDE.")
    if(args.server !== false) {
        let serverCfg      = Object.assign({},args)
        serverCfg.dir      = root
        serverCfg.fallback = '/assets/index.html'
        server             = await Server.create(serverCfg)
    }
    mainWindow = createWindow()
    mainWindow.on("close", (evt) => {
       if (hideInsteadOfQuit) {
           evt.preventDefault()
           mainWindow.hide()
       }
   })
}

let port = Server.DEFAULT_PORT
if      (server)    { port = server.port }
else if (args.port) { port = args.port }
let origin = `http://localhost:${port}`

function urlParamsFromObject(obj) {
    let params = []
    for (let key in obj) {
        let val = obj[key]
        params.push(`${key}=${val}`)
    }
    return params.join("&")
}

function createWindow() {
    let webPreferences     = secureWebPreferences()
    webPreferences.preload = path.join(root,'preload.js')

    let windowPreferences  = {
        webPreferences       : webPreferences,
        width                : windowCfg.width,
        height               : windowCfg.height,
        frame                : args.frame,
        devTools             : false,
        sandbox              : true,
        backgroundThrottling : false,
        transparent          : false,
        titleBarStyle        : 'default'
    }

    if (args.dev) {
        windowPreferences.devTools = true
    }

    if (args.frame === false && process.platform === 'darwin') {
        windowPreferences.titleBarStyle = 'hiddenInset'
    }

    if (args['background-throttling']) {
        windowPreferences.backgroundThrottling = true
    }

    if (args.vibrancy === true) {
        windowPreferences.vibrancy = 'fullscreen-ui'
    }

    const window = new Electron.BrowserWindow(windowPreferences)

    window.setMenuBarVisibility(false)

    if (args.dev) {
        window.webContents.openDevTools()
    }

    let urlCfg = {
        platform        : process.platform,
        frame           : args.frame,
        dark_theme      : Electron.nativeTheme.shouldUseDarkColors,
        high_contrast   : Electron.nativeTheme.shouldUseHighContrastColors,
        crashReportHost : args.crashReportHost,
    }

    if (args.project)    { urlCfg.project = args.project }
    if (args.entryPoint) { urlCfg.entry   = args.entryPoint }

    let params  = urlParamsFromObject(urlCfg)
    let address = `${origin}?${params}`

    console.log(`Loading the window address ${address}`)
    window.loadURL(address)
    return window
}

/// By default, Electron will automatically approve all permission requests unless the developer has
/// manually configured a custom handler. While a solid default, security-conscious developers might
/// want to assume the very opposite. Follow the link to learn more:
// https://www.electronjs.org/docs/tutorial/security#4-handle-session-permission-requests-from-remote-content
function setupPermissions() {
    Electron.session.defaultSession.setPermissionRequestHandler (
        (webContents,permission,callback) => {
            const url = webContents.getURL()
            console.error(`Unhandled permission request '${permission}'.`)
        }
    )
}



// ==============
// === Events ===
// ==============

Electron.app.on('activate', () => {
    if (process.platform === 'darwin') {
        mainWindow.show()
    }
})

Electron.app.on('ready', () => {
    if (args.version) {
        let indent     = ' '.repeat(4)
        let maxNameLen = 0
        for (let name in versionInfo) {
            if (name.length > maxNameLen) {
                maxNameLen = name.length
            }
        }

        console.log("Frontend:")
        for (let name in versionInfo) {
            let label   = capitalizeFirstLetter(name)
            let spacing = ' '.repeat(maxNameLen - name.length)
            console.log(`${indent}${label}:${spacing} ${versionInfo[name]}`)
        }

        console.log("")
        console.log("Backend:")
        backendVersion().then((backend) => {
            if (!backend) {
                console.log(`${indent}No backend available.`)
            } else {
                let lines = backend.split(/\r?\n/)
                for (let line of lines) {
                    console.log(`${indent}${line}`)
                }
            }
            process.exit()
        })
    } else if (args.info) {
        printDebugInfo()
    } else {
        if(args.window !== false) {
            main()
        }
    }
})

if (process.platform === 'darwin') {
    hideInsteadOfQuit = true
    Electron.app.on('before-quit', function() {
        hideInsteadOfQuit = false
    })
}



// =================
// === Shortcuts ===
// =================

Electron.app.on('web-contents-created', (webContentsCreatedEvent, webContents) => {
    webContents.on('before-input-event', (beforeInputEvent, input) => {
        const {code,alt,ctrl,shift,meta} = input
        if (ctrl && alt && shift && !meta && code === 'KeyI') {
            Electron.BrowserWindow.getFocusedWindow().webContents.toggleDevTools({mode:'detach'})
        }
        if (ctrl && alt && shift && !meta && code === 'KeyR') {
            Electron.BrowserWindow.getFocusedWindow().reload()
        }

        let cmd_q       =  meta && !ctrl && !alt && !shift && code === 'KeyQ'
        let ctrl_q      = !meta &&  ctrl && !alt && !shift && code === 'KeyQ'
        let alt_f4      = !meta && !ctrl &&  alt && !shift && code === 'F4'
        let ctrl_w      = !meta &&  ctrl && !alt && !shift && code === 'KeyW'
        let quit_on_mac = process.platform == 'darwin' && (cmd_q || alt_f4)
        let quit_on_win = process.platform == 'win32'  && (alt_f4 || ctrl_w)
        let quit_on_lin = process.platform == 'linux'  && (alt_f4 || ctrl_q || ctrl_w)
        let quit        = quit_on_mac || quit_on_win || quit_on_lin
        if (quit) { Electron.app.quit() }
    })
})



// =============================
// === Deprecations & Fixmes ===
// =============================

// FIXME Enable Metal backend on MacOS https://github.com/electron/electron/issues/22465

// TODO[WD] Windows vibrancy
// https://github.com/fstudio/clangbuilder/issues/39
// https://github.com/Microsoft/vscode/issues/32257
// https://github.com/arkenthera/electron-vibrancy/issues/21

// TODO[WD] Window corner radius
// https://github.com/electron/electron/issues/22542
