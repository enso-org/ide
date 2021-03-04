export class Visualization {
    constructor(api) {
        // These go before `api` assignment so the `undefined` is not emitted to IDE.
        // First we will give deriving type a chance to overwrite them, then IDE will
        // invoke `emitPreprocessorChange()` on this.
        this.__preprocessorCode__ = undefined
        this.__preprocessorModule__ = undefined

        this.dom = api.root()
        this.__api__ = api
    }

    emitPreprocessorChange() {
        console.trace("Will emit",this.preprocessorCode,this.preprocessorModule)
        this.__api__.emit_preprocessor_change(this.preprocessorCode,this.preprocessorModule)
    }

    getPreprocessorCode() {
        return this.__preprocessorCode__
    }

    setPreprocessorCode(code) {
        if (code !== this.preprocessorCode) {
            this.__preprocessorCode__ = code
            this.emitPreprocessorChange()
        }
    }

    getPreprocessorModule() {
        return this.__preprocessorModule__
    }

    setPreprocessorModule(module) {
        if (module !== this.preprocessorModule) {
            this.__preprocessorModule__ = module
            this.emitPreprocessorChange()
        }
    }

    // Meant to be used when both code and module need to be set as a single update.
    setPreprocessor(code,module) {
        if (code !== this.preprocessorCode || code !== this.preprocessorModule) {
            this.__preprocessorCode__ = code
            this.__preprocessorModule__ = module
            this.emitPreprocessorChange()
        }
    }
}

export function __Visualization__() {
    return Visualization
}
