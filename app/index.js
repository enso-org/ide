//import * as wasm from "./dist/wasm/basegl_examples"

let pfx = "run_example_"






// =================
// === Animation ===
// =================

function ease_in_out_cubic(t) {
    return t<.5 ? 4*t*t*t : 1 - (-2*t+2) * (-2*t+2) * (-2*t+2) / 2
}

function ease_in_out_quad(t) {
    return t<.5 ? 2*t*t : 1 - (-2*t+2)*(-2*t+2) / 2
}

function ease_out_quart(t) {
    return 1-(--t)*t*t*t
}



// ============
// === Math ===
// ============

function polar_to_cartesian(radius, angle_degrees) {
    let angle = (angle_degrees-90) * Math.PI / 180.0
    return {
        x : radius * Math.cos(angle),
        y : radius * Math.sin(angle)
    }
}



// ===========
// === SVG ===
// ===========

function new_svg(width, height, str) {
    return `
    <svg version="1.1" baseProfile="full" xmlns="http://www.w3.org/2000/svg"
         xmlns:xlink="http://www.w3.org/1999/xlink"
         height="${height}" width="${width}" viewBox="0 0 ${height} ${width}">
    ${str}
    </svg>`
}

function svg_arc(radius, end_angle){
    let start_angle = 0
    if (end_angle < 0) {
        start_angle = end_angle
        end_angle   = 0
    }
    let start       = polar_to_cartesian(radius, end_angle)
    let end         = polar_to_cartesian(radius, start_angle)
    let large_arc   = end_angle - start_angle <= 180 ? "0" : "1"
    return `M 0 0 L ${start.x} ${start.y} A ${radius} ${radius} 0 ${large_arc} 0 ${end.x} ${end.y}`
}



// =========================
// === ProgressIndicator ===
// =========================

let bg_color     = "#ffffff"
let loader_color = "#303030"

function new_loader_progress_indicator_svg() {
    let width        = 128
    let height       = 128
    let alpha        = 0.9
    let inner_radius = 48
    let outer_radius = 60
    let mid_radius = (inner_radius + outer_radius) / 2
    let bar_width = outer_radius - inner_radius

    return new_svg(width,height,`
        <defs>
            <g id="progress_bar">
                <circle fill="${loader_color}" r="${outer_radius}"                               />
                <circle fill="${bg_color}"     r="${inner_radius}"                               />
                <path   fill="${bg_color}"     opacity="${alpha}" id="progress_indicator_mask"   />
                <circle fill="${loader_color}" r="${bar_width/2}" id="progress_indicator_corner" />
                <circle fill="${loader_color}" r="${bar_width/2}" cy="-${mid_radius}"            />
            </g>
        </defs>
        <g transform="translate(${width/2},${height/2})">
            <g transform="rotate(0,0,0)" id="progress_indicator">
                <use xlink:href="#progress_bar"></use>
            </g>
        </g>
    `)
}

class ProgressIndicator {
    constructor() {
        let center = document.createElement('div')
        center.style.width          = '100%'
        center.style.height         = '100%'
        center.style.display        = 'flex'
        center.style.justifyContent = 'center'
        center.style.alignItems     = 'center'
        document.body.appendChild(center)

        let progress_bar_svg   = new_loader_progress_indicator_svg()
        let progress_bar       = document.createElement('div')
        progress_bar.innerHTML = progress_bar_svg
        center.appendChild(progress_bar)

        this.progress_indicator        = document.getElementById("progress_indicator")
        this.progress_indicator_mask   = document.getElementById("progress_indicator_mask")
        this.progress_indicator_corner = document.getElementById("progress_indicator_corner")

        this.set(0)
        this.set_opacity(0)
    }

    set(value) {
        let min_angle  = 0
        let max_angle  = 359
        let angle_span = max_angle - min_angle
        let mask_angle = (1-value)*angle_span - min_angle
        let corner_pos = polar_to_cartesian(54, -mask_angle)
        this.progress_indicator_mask.setAttribute("d", svg_arc(128, -mask_angle))
        this.progress_indicator_corner.setAttribute("cx", corner_pos.x)
        this.progress_indicator_corner.setAttribute("cy", corner_pos.y)
    }

    set_opacity(val) {
        this.progress_indicator.setAttribute("opacity",val)
    }

    set_rotation(val) {
        this.progress_indicator.setAttribute("transform",`rotate(${val},0,0)`)
    }
}



// ============
// === Main ===
// ============

let progress_indicator = new ProgressIndicator







function format_mb(bytes) {
   return Math.round(10 * bytes / (1042 * 1024)) / 10
}


class Loader {
    constructor(total_bytes) {
        this.total_bytes       = total_bytes
        this.received_bytes    = 0
        this.download_speed    = 0
        this.last_receive_time = performance.now()
        this.on_done = () => {}
    }

    value() {
        if (this.total_bytes == 0) {
            return 0.3
        } else {
            return this.received_bytes / this.total_bytes
        }
    }

    done() {
        return this.received_bytes == this.total_bytes
    }

    on_receive(new_bytes) {
        this.received_bytes += new_bytes
        let time      = performance.now()
        let time_diff = time - this.last_receive_time
        this.download_speed = new_bytes / time_diff
        this.last_receive_time = time

        let percent  = this.show_percentage_value()
        let speed    = this.show_download_speed()
        let received = this.show_received_bytes()
        console.log(`${percent}% (${received}) (${speed}).`)
        if (this.done()) { this.on_done() }
    }

    show_percentage_value() {
        return Math.round(100 * this.value())
    }

    show_total_bytes() {
        return `${format_mb(this.total_bytes)} MB`
    }

    show_received_bytes() {
        return `${format_mb(this.received_bytes)} MB`
    }

    show_download_speed() {
        return `${format_mb(1000 * this.download_speed)} MB/s`
    }

    input_stream() {
        let loader = this
        return new WritableStream({
             write(t) {
                 loader.on_receive(t.length)
             }
        })
    }
}


let incorrect_mime_type_warning = `
'WebAssembly.instantiateStreaming' failed because your server does not serve wasm with
'application/wasm' MIME type. Falling back to 'WebAssembly.instantiate' which is slower.
`


async function log_group_collapsed(msg,f) {
    console.groupCollapsed(msg)
    let out
    try {
        out = await f()
    } catch (error) {
        console.groupEnd()
        throw error
    }
    console.groupEnd()
    return out
}

async function run() {
    let wasm_imports_fetch = await fetch('wasm_imports.js')
    let wasm_fetch         = await fetch('dist/wasm/basegl_examples_bg.wasm')
    let wasm_imports_bytes = parseInt(wasm_imports_fetch.headers.get('Content-Length'))
    let wasm_bytes         = parseInt(wasm_fetch.headers.get('Content-Length'))
    let total_bytes        = wasm_imports_bytes + wasm_bytes

    if (Number.isNaN(total_bytes)) {
        console.error("Loader corrupted. Server is not configured to send the 'Content-Length'.")
        total_bytes = 0
    }
    let loader = new Loader(total_bytes)
    run_loader_indicator(loader)

    let download_size = loader.show_total_bytes();

    let download_info = `Downloading WASM binary and its dependencies (${download_size}).`

    await log_group_collapsed(download_info, async () => {

        loader.on_done = () => {
            console.groupEnd()
            console.log("Download finished. Finishing WASM compilation.")
        }



        wasm_imports_fetch.clone().body.pipeTo(loader.input_stream())
        wasm_fetch.clone().body.pipeTo(loader.input_stream())


        let wasm_imports_js = await wasm_imports_fetch.text()

        console.log("WASM dependencies loaded.")
        console.log("Starting online WASM compilation.")

        let out = Function("let exports = {};" + wasm_imports_js + ";return exports")()
        let imports = out.wasm_imports()

        let result = await WebAssembly.instantiateStreaming(wasm_fetch, imports).catch(e => {
            return wasm_fetch.then(r => {
                if (r.headers.get('Content-Type') != 'application/wasm') {
                    console.warn(`${incorrect_mime_type_warning} Original error:\n`, e)
                    return r.arrayBuffer()
                } else {
                    todo
                }
            })
            .then(bytes => WebAssembly.instantiate(bytes, imports))
        })
    })
    console.log("WASM Compiled.")
}

run()



function run_loader_indicator(loader) {
    let rotation = 0
    let alpha = 0
    function show_step() {
        progress_indicator.set_opacity(ease_in_out_quad(alpha))
        alpha += 0.02
        if (alpha > 1) {
            alpha = 1
        } else {
            window.requestAnimationFrame(show_step)
        }
    }
    window.requestAnimationFrame(show_step)


    function loading_step(timestamp) {
        let value = loader.value()
        if (value > 1) {
            value = 1
        }
        progress_indicator.set(value)
        progress_indicator.set_rotation(rotation)

        rotation += 6
        if (value < 1) {
            window.requestAnimationFrame(loading_step)
        }
    }
    window.requestAnimationFrame(loading_step)
}




//function hello_screen(msg) {
//    let names = []
//    for (let fn of Object.getOwnPropertyNames(wasm)) {
//        if (fn.startsWith(pfx)) {
//            let name = fn.replace(pfx,"")
//            names.push(name)
//        }
//    }
//
//    if(msg==="" || msg===null || msg===undefined) {
//        msg = ""
//    }
//    let newDiv     = document.createElement("div")
//    let newContent = document.createTextNode(msg + "Choose an example:")
//    let currentDiv = document.getElementById("app")
//    let ul         = document.createElement('ul')
//    newDiv.appendChild(newContent)
//    document.body.insertBefore(newDiv, currentDiv)
//    newDiv.appendChild(ul)
//
//    for (let name of names) {
//        let li       = document.createElement('li')
//        let a        = document.createElement('a')
//        let linkText = document.createTextNode(name)
//        ul.appendChild(li)
//        a.appendChild(linkText)
//        a.title = name
//        a.href  = "/" + name
//        li.appendChild(a)
//    }
//}
//
//function main() {
//    let target = window.location.href.split('/')[3]
//    if (target === "") {
//        hello_screen()
//    } else {
//        let fn_name = pfx + target
//        let fn      = wasm[fn_name]
//        if (fn) { fn() } else {
//            hello_screen("WASM function '" + fn_name + "' not found! ")
//        }
//    }
//}
//
//main()



//async function main() {
//    let response = await fetch('dist/wasm/basegl_examples.js')
//    const reader = response.body.getReader()
//    const total_bytes = +response.headers.get('Content-Length')
//
//    let received_bytes = 0 // received that many bytes at the moment
//    let chunks = [] // array of received binary chunks (comprises the body)
//    while(true) {
//        const {done, value} = await reader.read()
//
//        if (done) {
//          break
//        }
//
//        chunks.push(value)
//        received_bytes += value.length
//
//        console.log(`Received ${received_bytes} of ${total_bytes}`)
//    }
//
//    // Step 4: concatenate chunks into single Uint8Array
//    let chunksAll = new Uint8Array(received_bytes) // (4.1)
//    let position = 0
//    for(let chunk of chunks) {
//      chunksAll.set(chunk, position) // (4.2)
//      position += chunk.length
//    }
//
//    // Step 5: decode into a string
//    let result = new TextDecoder("utf-8").decode(chunksAll)
//
//    // We're done!
//    console.log("DONE")
////    console.log(result)
//    let out = Function(result)()
//    console.log(out)
//    out.init('dist/wasm/basegl_examples_bg.wasm')
////    let commits = JSON.parse(result)
////    alert(commits[0].author.login)
//
//}
//
//main()
//