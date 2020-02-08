//import * as wasm from "basegl"
import * as wasm from "./dist/wasm/basegl_examples"

let pfx = "run_example_"



function format_mb(bytes) {
   return Math.round(10 * bytes / (1042 * 1024)) / 10
}

async function run() {
    let imports             = wasm.get_imports()
    let response            = await fetch('dist/wasm/basegl_examples_bg.wasm')
    let wasm_total_bytes    = response.headers.get('Content-Length')
    let wasm_received_bytes = 0
    let last_receive_time   = performance.now()

    console.groupCollapsed(`Loading WASM (${format_mb(wasm_total_bytes)} MB).`)

    response.clone().body.pipeTo(
        new WritableStream({
             write(t) {
                 let new_bytes = t.length
                 wasm_received_bytes += new_bytes
                 let time      = performance.now()
                 let time_diff = time - last_receive_time
                 let percent   = Math.round(100 * wasm_received_bytes / wasm_total_bytes)
                 let speed     = `${format_mb(1000 * new_bytes / time_diff)} MB/s`
                 let received  = `${format_mb(wasm_received_bytes)} MB`
                 last_receive_time = time
                 console.log(`${percent}% (${received}) (${speed}).`)
                 if (wasm_received_bytes == wasm_total_bytes) {
                     console.groupEnd()
                     console.log("Compiling WASM.")
                 }
             }
        })
    )

    let result = await WebAssembly.instantiateStreaming(response, imports).catch(e => {
        return response
        .then(r => {
            if (r.headers.get('Content-Type') != 'application/wasm') {
                console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e)
                return r.arrayBuffer()
            } else {
                throw e
            }
        })
        .then(bytes => WebAssembly.instantiate(bytes, imports))
    })

    console.log("WASM Compiled.")
}

run()

var center = document.createElement('div')
center.style.width          = '100%'
center.style.height         = '100%'
center.style.display        = 'flex'
center.style.justifyContent = 'center'
center.style.alignItems     = 'center'
document.body.appendChild(center)

let svg = `
<svg version="1.1" baseProfile="full" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" height="128" width="128" viewBox="0 0 128 128">
  <defs>
    <circle id="innerCircle" cx="32" cy="32" r="20.816326530612244"></circle>
    <circle id="leftAtom" cx="17.591836734693878" cy="32" r="14.408163265306122"></circle>
    <circle id="rightAtom" cx="42.40816326530612" cy="32" r="10.408163265306122"></circle>
    <mask id="innerCircleMask">
      <use xlink:href="#innerCircle" fill="white"></use>
    </mask>

    <rect id="bg" width="64" height="64" fill="white"></rect>

    <mask id="mainShapeMask">
      <use xlink:href="#bg"></use>
      <use xlink:href="#leftAtom" fill="black"></use>
      <rect cy="32" width="64" height="32" fill="black"></rect>
    </mask>

    <g id="front">
      <use xlink:href="#innerCircle" mask="url(#mainShapeMask)"></use>
      <use xlink:href="#rightAtom"></use>
    </g>

    <g id="logo">
    <use xlink:href="#border"></use>
      <use xlink:href="#front" transform="rotate(35 32 32)"></use>
    </g>

    </defs>


    <g transform="scale(2)"><use xlink:href="#logo" fill="#252525"></use></g>
</svg>
`

let progress_bar_svg = `
<svg version="1.1" baseProfile="full" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" height="128" width="128" viewBox="0 0 128 128">
    <defs>
        <g id="progress_bar">
            <circle cx="64" cy="64" r="60" fill="#252525" />
            <circle cx="64" cy="64" r="48" fill="white" />
            <path id="load_progress_indicator" fill="#FFFFFFDD" />
            <circle cx="64" cy="10" r="6" fill="#252525" />
            <circle id="load_progress_indicator_corner" cx="64" cy="10" r="6" fill="#252525" />
        </g>
    </defs>
    <g id="gg" transform="rotate(45,64,64)"><use xlink:href="#progress_bar" fill="#252525"></use></g>
</svg>

`

var logo = document.createElement('div')
logo.style.position = "absolute"
logo.style.zIndex = 10
logo.innerHTML = svg
//center.appendChild(logo)

var progress_bar = document.createElement('div')
//progress_bar.style.position = "absolute"
progress_bar.innerHTML = progress_bar_svg
center.appendChild(progress_bar)


function polarToCartesian(centerX, centerY, radius, angleInDegrees) {
  var angleInRadians = (angleInDegrees-90) * Math.PI / 180.0

  return {
    x: centerX + (radius * Math.cos(angleInRadians)),
    y: centerY + (radius * Math.sin(angleInRadians))
  }
}

function describeArc(x, y, radius, startAngle, endAngle){

    var start = polarToCartesian(x, y, radius, endAngle)
    var end = polarToCartesian(x, y, radius, startAngle)

    var largeArcFlag = endAngle - startAngle <= 180 ? "0" : "1"

    var d = [
        "M", 64, 64,
        "L", start.x, start.y,
        "A", radius, radius, 0, largeArcFlag, 0, end.x, end.y
    ].join(" ")

    return d
}

let load_progress_indicator        = document.getElementById("load_progress_indicator");
let load_progress_indicator_corner = document.getElementById("load_progress_indicator_corner");

function set_progress(value) {
    let angle = (1-value)*359
    let corner_pos = polarToCartesian(64, 64, 54, angle)
    load_progress_indicator.setAttribute("d", describeArc(64, 64, 128, 0, angle))
    load_progress_indicator_corner.setAttribute("cx", corner_pos.x)
    load_progress_indicator_corner.setAttribute("cy", corner_pos.y)
}

let foo = polarToCartesian(64, 64, 54, 130)
document.getElementById("load_progress_indicator").setAttribute("d", describeArc(64, 64, 128, 0, 130))
document.getElementById("load_progress_indicator_corner").setAttribute("cx", foo.x)
document.getElementById("load_progress_indicator_corner").setAttribute("cy", foo.y)

let gg = document.getElementById("gg")

let rotation = 0

let value = 0

function loading_step(timestamp) {
    value += 0.003
    if (value > 1) {
        value = 1
    }
    rotation += 6

    let angle = (1-value)*359
    let foo = polarToCartesian(64, 64, 54, angle)
    document.getElementById("load_progress_indicator").setAttribute("d", describeArc(64, 64, 128, 0, angle))
    document.getElementById("load_progress_indicator_corner").setAttribute("cx", foo.x)
    document.getElementById("load_progress_indicator_corner").setAttribute("cy", foo.y)

    gg.setAttribute("transform", `rotate(${rotation},64,64)`)
    window.requestAnimationFrame(loading_step)
}

window.requestAnimationFrame(loading_step)




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