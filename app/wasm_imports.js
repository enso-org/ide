import * as wasm from "./dist/wasm/basegl_examples"

exports.wasm_imports = function() {
    return wasm.get_imports()
}
