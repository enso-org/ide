#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! regex = "1.3.4"
//! ```

extern crate regex;

use std::process::Command;
use std::fs;
use regex::Regex;

fn main() {

    println!("Hello, World!");
    Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--no-typescript")
        .arg("--out-dir")
        .arg("../../target/web")
        .arg("lib/gui")
        .status()
        .expect("failed to execute process");

    fs::rename("target/web/gui_bg.wasm","target/web/gui.wasm").unwrap();

    let code = fs::read_to_string("target/web/gui.js").expect("Cannot read the file.");

    let pattern = Regex::new(r"(?ms)if \(\(typeof URL.*}\);").unwrap();
    let code    = pattern.replace_all(&code, "return imports");

    let pattern = Regex::new(r"(?ms)if \(typeof module.*let result").unwrap();
    let code    = pattern.replace_all(&code, "let result");

    let code = format!("{}\nexport function after_load(w,m) {{ wasm = w; init.__wbindgen_wasm_module = m;}}",code);

    fs::write("target/web/gui.js",code);


    Command::new("gzip")
        .arg("--keep")
        .arg("--best")
        .arg("--force")
        .arg("target/web/gui.wasm")
        .status()
        .expect("failed to execute process");

    // TODO
    // && rm -Rf app/src-rust-gen
    // && cp -R target/web app/src-rust-gen
}
