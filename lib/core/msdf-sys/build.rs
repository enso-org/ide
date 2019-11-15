extern crate download_lp;

mod msdfgen_wasm {

    pub const VERSION     : &str = "v1.0";
    pub const FILENAME    : &str = "msdfgen_wasm.js";
    pub const PROJECT_URL : &str = "https://github.com/luna/msdfgen-wasm";

    pub fn download() {
        let url = format!(
            "{project}/releases/download/{version}/{filename}",
            project  = PROJECT_URL,
            version  = VERSION,
            filename = FILENAME
        );

        if std::path::Path::new(FILENAME).exists() {
            std::fs::remove_file(FILENAME).unwrap();
        }

        // Note [Downloading to src dir]
        download_lp::download(url.as_str(), ".").unwrap();
    }
}

/* Note [Downloading to src dir]
 * In theory, build.rs scripts should create and modify files in OUT_DIR only, but I haven't found
 * any way to make #[wasm_bindgen(module="")] taking a file from OUT_DIR (except by providing a full
 * system path, which is obviously awful)
 *
 * Thanks for your understanding
 *
 * If you find and implement a better way to downloading js snippets, please remember to remove
 * msdfgen_wasm.js entry from .gitignore
 */

fn main() {
    msdfgen_wasm::download();
    println!("cargo:rerun-if-changed=build.rs");
}

