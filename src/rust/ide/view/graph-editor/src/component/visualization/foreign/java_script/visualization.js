export class Visualization {
    constructor(api) {
        this.dom = api.root()
        this.__api__ = api
    }
    setPreprocessor(code,module) {
        console.debug(`Passing to rust: setPreprocessor(${code},${module})`)
        this.__api__.emit_preprocessor_change(code,module)
    }
}

export function __Visualization__() {
    return Visualization
}
