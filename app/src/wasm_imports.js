import * as wasm_rust_glue from "wasm_rust_glue"

exports.wasm_imports = function() {
    return wasm_rust_glue.default()
}

exports.after_load = function (w, module) {
    return wasm_rust_glue.after_load(w,module)
}
