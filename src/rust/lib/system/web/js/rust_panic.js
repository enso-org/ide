class RustPanic extends Error {
    constructor(message) {
        super(message)
        this.name = 'RustPanic'
    }
}

export function throw_panic_error(message) {
    let error = new RustPanic(message);
    // Set `error.stack` to the current stack trace, up to but not including this call to `throw_panic_error`
    Error.captureStackTrace(error, throw_panic_error);
    throw error
}