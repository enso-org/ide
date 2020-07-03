/// This module is responsible for loading the WASM binary, its dependencies, and providing the
/// user with a visual representation of this process (welcome screen). It also implements a view
/// allowing to choose a debug rendering test from.

import * as loader_module from 'enso-studio-common/src/loader'
import * as html_utils    from 'enso-studio-common/src/html_utils'
import * as animation     from 'enso-studio-common/src/animation'



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
/// with `{no_loader:true}` option.
async function download_content(cfg) {
    let wasm_glue_fetch = await fetch('/assets/wasm_imports.js')
    let wasm_fetch      = await fetch('/assets/ide.wasm')
    let loader          = new loader_module.Loader([wasm_glue_fetch,wasm_fetch],cfg)

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
let main_scene_name = 'ide'

/// Prefix name of each scene defined in the WASM binary.
let wasm_fn_pfx = "run_example_"


/// Displays a debug screen which allows the user to run one of predefined debug examples.
function show_debug_screen(wasm,msg) {
    let names = []
    for (let fn of Object.getOwnPropertyNames(wasm)) {
        if (fn.startsWith(wasm_fn_pfx)) {
            let name = fn.replace(wasm_fn_pfx,"")
            names.push(name)
        }
    }

    if(msg==="" || msg===null || msg===undefined) { msg = "" }
    let debug_screen_div = html_utils.new_top_level_div()
    let newDiv     = document.createElement("div")
    let newContent = document.createTextNode(msg + "Choose an example:")
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
        a.href    = "javascript:{}"
        a.onclick = () => {
            html_utils.remove_node(debug_screen_div)
            let fn_name = wasm_fn_pfx + name
            let fn = wasm[fn_name]
            fn()
        }
        li.appendChild(a)
    }
}



// ========================
// === Main Entry Point ===
// ========================

let root = document.getElementById('root')

function prepare_root(cfg) {
    root.style.backgroundColor = '#e8e7e699'
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

/// Main entry point. Loads WASM, initializes it, chooses the scene to run.
async function main() {
    disableContextMenu()
    let target = window.location.pathname.split('/')
    target.splice(0,1)
    let cfg = getUrlParams()
    prepare_root(cfg)

    let debug_mode    = target[0] == "debug"
    let debug_target  = target[1]
    let no_loader     = debug_mode && debug_target

    await windowShowAnimation()
    let {wasm,loader} = await download_content({no_loader})

    if (debug_mode) {
        loader.destroy()
        if (debug_target) {
            let fn_name = wasm_fn_pfx + debug_target
            let fn      = wasm[fn_name]
            if (fn) { fn() } else {
                show_debug_screen(wasm,"WASM function '" + fn_name + "' not found! ")
            }
        } else {
            show_debug_screen(wasm)
        }
    } else {
        wasm[wasm_fn_pfx + main_scene_name]()
    }
}

main()
