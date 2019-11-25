use basegl_build_utilities::github_download;

mod msdfgen_wasm {
    use crate::github_download;
    use std::path;

    pub const VERSION     : &str = "v1.0.1";
    pub const FILENAME    : &str = "msdfgen_wasm.js";
    pub const PROJECT_URL : &str = "https://github.com/luna/msdfgen-wasm";

    pub fn download() {
        github_download(
            PROJECT_URL,
            VERSION,
            FILENAME,
            path::Path::new(".") // Note [Downloading to src dir]
        )
    }

    /* Note [Downloading to src dir]
     * In theory, build.rs scripts should create and modify files in OUT_DIR
     * only, but I haven't found any way to make #[wasm_bindgen(module="")]
     * taking a file from OUT_DIR (except by providing a full system path,
     * which is obviously awful)
     *
     * Thanks for your understanding
     *
     * If you find and implement a better way to downloading js snippets, please
     * remember to remove msdfgen_wasm.js entry from .gitignore
     */
}

fn main() {
    msdfgen_wasm::download();
    println!("cargo:rerun-if-changed=build.rs");
}
