export class Visualization {
    constructor(api) {
        // These go before `api` assignment so the `undefined` is not emitted to IDE.
        // First we will give deriving type a chance to overwrite them, then IDE will
        // invoke `emitPreprocessorChange()` on this.
        this.__preprocessorCode__ = null
        this.__preprocessorModule__ = null

        this.dom = api.root()
        this.__api__ = api
    }

    __emitPreprocessorChange__() {
        this.__api__.emit_preprocessor_change(
            this.__preprocessorCode__,
            this.__preprocessorModule__
        )
    }

    getPreprocessorCode() {
        return this.__preprocessorCode__
    }

    setPreprocessorCode(code) {
        if (code !== this.__preprocessorCode__) {
            this.__preprocessorCode__ = code
            this.__emitPreprocessorChange__()
        }
    }

    getPreprocessorModule() {
        return this.__preprocessorModule__
    }

    setPreprocessorModule(module) {
        if (module !== this.__preprocessorModule__) {
            this.__preprocessorModule__ = module
            this.__emitPreprocessorChange__()
        } else {
            console.error(
                'skipping, as',
                module,
                ' === ',
                this.__preprocessorModule__
            )
        }
    }

    // Meant to be used when both code and module need to be set as a single update.
    setPreprocessor(code, module) {
        if (
            code !== this.__preprocessorCode__ ||
            code !== this.__preprocessorModule__
        ) {
            this.__preprocessorCode__ = code
            this.__preprocessorModule__ = module
            this.__emitPreprocessorChange__()
        }
    }
}

export function __Visualization__() {
    return Visualization
}
