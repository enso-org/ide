export class Visualization {
    constructor(api) {
        this.dom = api.root()
        this.__api__ = api
        this.__preprocessorCode__ = undefined
        this.__moduleContext__ = undefined
    }

    updatePreprocessor() {
        let code = this.__preprocessorCode__
        let module = this.__moduleContext__
        this.__api__.emit_preprocessor_change(code,module)
    }

    getPreprocessorCode() {
        this.__preprocessorCode__
    }

    setPreprocessorCode(code) {
        this.__preprocessorCode__ = code
    }

    setPreprocessor(code,module) {
        console.debug(`Passing to rust: setPreprocessor(${code},${module})`)
        this.__api__.emit_preprocessor_change(code,module)
    }
}

export function __Visualization__() {
    return Visualization
}
