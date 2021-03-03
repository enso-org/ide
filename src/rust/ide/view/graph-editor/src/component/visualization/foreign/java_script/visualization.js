export class Visualization {
    constructor(api) {
        this.dom = api.root()
        this.__api__ = api
        this.__preprocessorCode__ = undefined
        this.__preprocessorModule__ = undefined
    }

    emitPreprocessorChange() {
        this.__api__.emit_preprocessor_change(this.preprocessorCode,this.preprocessorModule)
    }

    get preprocessorCode() {
        return this.__preprocessorCode__
    }

    set preprocessorCode(code) {
        if (code !== this.preprocessorCode) {
            this.__preprocessorCode__ = code
            this.emitPreprocessorChange()
        }
    }

    get preprocessorModule() {
        return this.__preprocessorModule__
    }

    set preprocessorModule(module) {
        if (module !== this.preprocessorModule) {
            this.__preprocessorModule__ = module
            this.emitPreprocessorChange()
        }
    }

    // Meant to be used when both code and module need to be set as a single update.
    // Otherwise `preprocessorCode` and `preprocessorModule` accessors should be preferred.
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
