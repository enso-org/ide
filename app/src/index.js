/// This module is responsible for loading the WASM binary, its dependencies, and providing the
/// user with a visual representation of this process (welcome screen). It also implements a view
/// allowing to choose a debug rendering test from.

import * as loader_module from './loader'
import * as html_utils    from './html_utils'



// ========================
// === Content Download ===
// ========================

let incorrect_mime_type_warning = `
'WebAssembly.instantiateStreaming' failed because your server does not serve wasm with
'application/wasm' MIME type. Falling back to 'WebAssembly.instantiate' which is slower.
`

async function download_content(cfg) {
    let wasm_imports_fetch = await fetch('/assets/wasm_imports.js')
    let wasm_fetch         = await fetch('/assets/gui.wasm')
    let wasm_imports_bytes = parseInt(wasm_imports_fetch.headers.get('Content-Length'))
    let wasm_bytes         = parseInt(wasm_fetch.headers.get('Content-Length'))
    let total_bytes        = wasm_imports_bytes + wasm_bytes

    if (Number.isNaN(total_bytes)) {
        console.error("Loader corrupted. Server is not configured to send the 'Content-Length'.")
        total_bytes = 0
    }
    let loader = new loader_module.Loader(total_bytes, cfg)

    loader.on_done = () => {
        console.groupEnd()
        console.log("Download finished. Finishing WASM compilation.")
    }

    let download_size = loader.show_total_bytes();
    let download_info = `Downloading WASM binary and its dependencies (${download_size}).`
    let wasm_loader   = html_utils.log_group_collapsed(download_info, async () => {
        wasm_imports_fetch.clone().body.pipeTo(loader.input_stream())
        wasm_fetch.clone().body.pipeTo(loader.input_stream())


        let wasm_imports_js = await wasm_imports_fetch.text()

        console.log("WASM dependencies loaded.")
        console.log("Starting online WASM compilation.")

        let out = Function("let exports = {};" + wasm_imports_js + ";return exports")()
        let imports = out.wasm_imports()

        let wasm_loader = await WebAssembly.instantiateStreaming(wasm_fetch, imports).catch(e => {
            return wasm_fetch.then(r => {
                if (r.headers.get('Content-Type') != 'application/wasm') {
                    console.warn(`${incorrect_mime_type_warning} Original error:\n`, e)
                    return r.arrayBuffer()
                } else {
                    throw("Server not configured to serve WASM with 'application/wasm' mime type.")
                }
            })
            .then(bytes => WebAssembly.instantiate(bytes, imports))
        })

        wasm_loader.out = out
        return wasm_loader
    })

    let {wasm,module,out} = await wasm_loader.then(({instance, module, out}) => {
        let wasm = instance.exports;
        // init.__wbindgen_wasm_module = module;
        return {wasm,module,out};
    });

    console.log("WASM Compiled.")
    out.after_load(wasm,module)

    await loader.initialized
    return {wasm,loader}
}



// ====================
// === Debug Screen ===
// ====================

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

/// Main entry point. Loads WASM, initializes it, chooses the scene to run.
async function main() {
    let target = window.location.href.split('/')
    target.splice(0,3)

    let debug_mode    = target[0] == "debug"
    let debug_target  = target[1]
    let no_animation  = debug_mode && debug_target
    let {wasm,loader} = await download_content({no_animation})
//    loader.destroy()

    if (debug_mode) {
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
        wasm[wasm_fn_pfx + 'shapes']()
    }
}

main()
