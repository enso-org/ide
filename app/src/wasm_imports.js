import * as wasm_rust_glue from "wasm_rust_glue"

exports.wasm_imports = function() {
    return wasm_rust_glue.default()
}
