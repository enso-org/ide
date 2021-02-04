/// This module is responsible for loading the WASM binary, its dependencies, and providing the
/// user with a visual representation of this process (welcome screen). It also implements a view
/// allowing to choose a debug rendering test from.

import * as loader_module from 'enso-studio-common/src/loader'
import * as html_utils    from 'enso-studio-common/src/html_utils'
import * as animation     from 'enso-studio-common/src/animation'
import * as globalConfig  from '../../../../config.yaml'



// ==================
// === Global API ===
// ==================

let API = {}
window[globalConfig.windowAppScopeName] = API

import cfg from '../../../config'



// ========================
// === Content Download ===
// ========================

let incorrect_mime_type_warning = `
'WebAssembly.instantiateStreaming' failed because your server does not serve wasm with
'application/wasm' MIME type. Falling back to 'WebAssembly.instantiate' which is slower.
`

function wasm_instantiate_streaming(resource,imports) {
    return WebAssembly.instantiateStreaming(resource,imports).catch(e => {
        return wasm_fetch.then(r => {
            if (r.headers.get('Content-Type') != 'application/wasm') {
                console.warn(`${incorrect_mime_type_warning} Original error:\n`, e)
                return r.arrayBuffer()
            } else {
                throw("Server not configured to serve WASM with 'application/wasm' mime type.")
            }
        }).then(bytes => WebAssembly.instantiate(bytes,imports))
    })
}


/// Downloads the WASM binary and its dependencies. Displays loading progress bar unless provided
/// with `{use_loader:false}` option.
async function download_content(config) {
    let wasm_glue_fetch = await fetch(config.wasm_glue_url)
    let wasm_fetch      = await fetch(config.wasm_url)
    let loader =
        new loader_module.Loader([wasm_glue_fetch,wasm_fetch], config)

    loader.done.then(() => {
        console.groupEnd()
        console.log("Download finished. Finishing WASM compilation.")
    })

    let download_size = loader.show_total_bytes()
    let download_info = `Downloading WASM binary and its dependencies (${download_size}).`
    let wasm_loader   = html_utils.log_group_collapsed(download_info, async () => {
        let wasm_glue_js = await wasm_glue_fetch.text()
        let wasm_glue    = Function("let exports = {};" + wasm_glue_js + "; return exports")()
        let imports      = wasm_glue.wasm_imports()
        console.log("WASM dependencies loaded.")
        console.log("Starting online WASM compilation.")
        let wasm_loader       = await wasm_instantiate_streaming(wasm_fetch,imports)
        wasm_loader.wasm_glue = wasm_glue
        return wasm_loader
    })

    let wasm = await wasm_loader.then(({instance,module,wasm_glue}) => {
        let wasm = instance.exports
        wasm_glue.after_load(wasm,module)
        return wasm
    })
    console.log("WASM Compiled.")

    await loader.initialized
    return {wasm,loader}
}



// ====================
// === Debug Screen ===
// ====================

/// The name of the main scene in the WASM binary.
let main_entry_point = 'ide'

/// Prefix name of each scene defined in the WASM binary.
let wasm_entry_point_pfx = "entry_point_"


/// Displays a debug screen which allows the user to run one of predefined debug examples.
function show_debug_screen(wasm,msg) {
    mixpanel.track("show_debug_screen")
    let names = []
    for (let fn of Object.getOwnPropertyNames(wasm)) {
        if (fn.startsWith(wasm_entry_point_pfx)) {
            let name = fn.replace(wasm_entry_point_pfx,"")
            names.push(name)
        }
    }

    if(msg==="" || msg===null || msg===undefined) { msg = "" }
    let debug_screen_div = html_utils.new_top_level_div()
    let newDiv     = document.createElement("div")
    let newContent = document.createTextNode(msg + "Available entry points:")
    let currentDiv = document.getElementById("app")
    let ul         = document.createElement('ul')
    debug_screen_div.style.position = 'absolute'
    debug_screen_div.style.zIndex   = 1
    newDiv.appendChild(newContent)
    debug_screen_div.appendChild(newDiv)
    newDiv.appendChild(ul)

    for (let name of names) {
        let li       = document.createElement('li')
        let a        = document.createElement('a')
        let linkText = document.createTextNode(name)
        ul.appendChild(li)
        a.appendChild(linkText)
        a.title   = name
        a.href    = "?entry="+name
        li.appendChild(a)
    }
}



// ====================
// === Scam Warning ===
// ====================

function printScamWarning() {
    let headerCSS = `
        color : white;
        background : crimson;
        display : block;
        border-radius : 8px;
        font-weight : bold;
        padding: 10px 20px 10px 20px;
    `
    let headerCSS1 = headerCSS + "font-size : 46px;"
    let headerCSS2 = headerCSS + "font-size : 20px;"
    let msgCSS     = "font-size:16px;"

    let msg1 = "This is a browser feature intended for developers. If someone told you to " +
               "copy-paste something here, it is a scam and will give them access to your " +
               "account and data."
    let msg2 = "See https://github.com/enso-org/ide/blob/main/docs/security/selfxss.md for more " +
               "information."
    console.log("%cStop!",headerCSS1)
    console.log("%cYou may be victim of a scam!",headerCSS2)
    console.log("%c"+msg1,msgCSS)
    console.log("%c"+msg2,msgCSS)
}



// ======================
// === Remote Logging ===
// ======================

const mixpanel = require('mixpanel-browser');
mixpanel.init("5b541aeab5e08f313cdc1d1bbebc12ac", { "api_host": "https://api-eu.mixpanel.com" }, "");

const MAX_MESSAGE_LENGTH = 500;

function remoteLog(event,data) {
    if (mixpanel) {
        event = JSON.stringify(event).substr(0,MAX_MESSAGE_LENGTH)
        if (data !== undefined) {
            data = JSON.stringify(data).substr(0,MAX_MESSAGE_LENGTH)
        }
        mixpanel.track(event,{data:data});
    } else {
        console.warn(`Failed to log the event '${event}'.`)
    }
}

window.enso.remote_log = remoteLog

window.setInterval(() =>{remoteLog("alive");}, 1000 * 60)


// ======================
// === Logs Buffering ===
// ======================

const logsFns = ['log','info','debug','warn','error','group','groupCollapsed','groupEnd']

class LogRouter {
    constructor() {
        this.buffer    = []
        this.raw       = {}
        this.autoFlush = true
        console.autoFlush = true
        for (let name of logsFns) {
            this.raw[name] = console[name]
            console[name] = (...args) => {
                this.handle(name,args)
            }
        }
    }

    auto_flush_on() {
        this.autoFlush = true
        console.autoFlush = true
        for (let {name,args} of this.buffer) {
            this.raw[name](...args)
        }
        this.buffer = []
    }

    handle(name,args) {
        if (this.autoFlush) {
            this.raw[name](...args)
        } else {
            this.buffer.push({name,args})
        }

        // The following code is just a hack to discover if the logs start with `[E]` which
        // indicates errors from Rust logger.
        if (name == 'error') {
            this.handleError(...args)
        } else if (name == 'log') {
            let firstArg = args[0]
            if (firstArg !== undefined && firstArg.startsWith("%c")) {
                let firstArgBody = firstArg.slice(2);
                let bodyStartIndex = firstArgBody.indexOf("%c");
                if (bodyStartIndex !== -1) {
                    let body = firstArgBody.slice(bodyStartIndex + 3);
                    let is_error = body.startsWith("[E]");
                    if (is_error) {
                        this.handleError(body)
                    }
                }
            }
        }
    }

    handleError(...args) {
        remoteLog(args)
    }
}

let logRouter = new LogRouter()

function hideLogs() {
    console.log('All subsequent logs will be hidden. Eval `showLogs()` to reveal them.')
    logRouter.autoFlush = false
    console.autoFlush = false
}

function showLogs() {
    logRouter.auto_flush_on()
}

window.showLogs = showLogs



// ======================
// === Crash Handling ===
// ======================

function initCrashHandling() {
    setupCrashDetection()
    if (previousCrashMessageExists()) {
        showCrashBanner(getPreviousCrashMessage())
        clearPreviousCrashMessage()
    }
}

const crashMessageStorageKey = "crash-message"

function previousCrashMessageExists() {
    return sessionStorage.getItem(crashMessageStorageKey) !== null
}

function getPreviousCrashMessage() {
    return sessionStorage.getItem(crashMessageStorageKey)
}

function storeLastCrashMessage(message) {
    sessionStorage.setItem(crashMessageStorageKey, message)
}

function clearPreviousCrashMessage() {
    sessionStorage.removeItem(crashMessageStorageKey)
}


// === Crash detection ===

function setupCrashDetection() {
    // This will only have an effect if the GUI is running in V8.
    // (https://v8.dev/docs/stack-trace-api#compatibility)
    Error.stackTraceLimit = 100

    window.addEventListener('error', function (event) {
        // We prefer stack traces over plain error messages but not all browsers produce traces.
        handleCrash(event.error.stack || event.message)
    })
    window.addEventListener('unhandledrejection', function (event) {
        // As above, we prefer stack traces.
        // But here, `event.reason` is not even guaranteed to be an `Error`.
        handleCrash(event.reason.stack || event.reason.message || "Unhandled rejection")
    })
}

function handleCrash(message) {
    remoteLog("crash", message)
    if (document.getElementById(crashBannerId) === null) {
        storeLastCrashMessage(message)
        location.reload()
    } else {
        for (let element of [... document.body.childNodes]) {
            if (element.id !== crashBannerId) {
                element.remove()
            }
        }
        document.getElementById(crashBannerContentId).insertAdjacentHTML("beforeend",
            `<hr>
             <div>A second error occurred. This time, the IDE will not automatically restart.</div>`)
    }
}


// === Crash recovery ===

// Those IDs should be the same that are used in index.html.
const crashBannerId = "crash-banner"
const crashBannerContentId = "crash-banner-content"
const crashReportButtonId = "crash-report-button"
const crashBannerCloseButtonId = "crash-banner-close-button"

function showCrashBanner(message) {
    document.body.insertAdjacentHTML('afterbegin',
        `<div id="${crashBannerId}">
            <button id="${crashBannerCloseButtonId}" class="icon-button">✖</button>
            <div id="${crashBannerContentId}">
                <button id="${crashReportButtonId}">Report</button>
                An internal error occurred and the Enso IDE has been restarted.
            </div>
        </div>`
    )

    const banner = document.getElementById(crashBannerId)
    const content = document.getElementById(crashBannerContentId)
    const report_button = document.getElementById(crashReportButtonId)
    const close_button = document.getElementById(crashBannerCloseButtonId)

    report_button.onclick = async _event => {
        try {
            await reportCrash(message)
            content.textContent = "Thank you, the crash was reported."
        } catch (e) {
            content.textContent = "The crash could not be reported."
        }
    }
    close_button.onclick = () => {
        banner.remove()
    }
}

async function reportCrash(message) {
    const crashReportHost = getUrlParams().crashReportHost || cfg.defaultLogServerHost
    await fetch(`http://${crashReportHost}/`, {
        method: 'POST',
        mode: 'no-cors',
        headers: {
          'Content-Type': 'text/plain'
        },
        body: message
      })
}



// ========================
// === Main Entry Point ===
// ========================

function style_root() {
    let root = document.getElementById('root')
    root.style.backgroundColor = '#f6f3f199'
}

function getUrlParams() {
    let url    = window.location.search
    let query  = url.substr(1)
    let result = {}
    query.split("&").forEach(function(part) {
        let item = part.split("=")
        result[item[0]] = decodeURIComponent(item[1])
    })
    return result
}

/// Waits for the window to finish its show animation. It is used when the website is run in
/// Electron. Please note that it returns immediately in the web browser.
async function windowShowAnimation() {
    await window.showAnimation
}

function disableContextMenu() {
    document.body.addEventListener('contextmenu', e => {
        e.preventDefault()
    })
}

function ok(value) {
    return value !== null && value !== undefined
}

/// Main entry point. Loads WASM, initializes it, chooses the scene to run.
API.main = async function (inputConfig) {
    remoteLog("main")
    let defaultConfig = {
        use_loader    : true,
        wasm_url      : '/assets/ide.wasm',
        wasm_glue_url : '/assets/wasm_imports.js'
    }
    let urlParams = new URLSearchParams(window.location.search);
    let urlConfig = Object.fromEntries(urlParams.entries())
    let config    = Object.assign(defaultConfig,inputConfig,urlConfig)
    API[globalConfig.windowAppScopeConfigName] = config

    initCrashHandling()
    style_root()
    printScamWarning()
    hideLogs()
    disableContextMenu()

    let entryTarget = ok(config.entry) ? config.entry : main_entry_point
    config.use_loader = config.use_loader && (entryTarget === main_entry_point)

    remoteLog("window_show_animation")
    await windowShowAnimation()
    remoteLog("download_content")
    let {wasm,loader} = await download_content(config)
    remoteLog("wasm_loaded")
    if (entryTarget) {
        let fn_name = wasm_entry_point_pfx + entryTarget
        let fn      = wasm[fn_name]
        if (fn) { fn() } else {
            loader.destroy()
            show_debug_screen(wasm,"Unknown entry point '" + entryTarget + "'. ")
        }
    } else {
        show_debug_screen(wasm)
    }
}
