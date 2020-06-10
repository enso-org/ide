export class Visualization {
    constructor(init) {
        this.dom = init.root();
        this.inner = init;
    }
    setPreprocessor (code) {
        this.inner.emit_preprocessor_change(code)
    }
}

export function cls() {
    return Visualization
}
