#![feature(option_result_contains)]

//! This build script is responsible for ensuring that if parser targets wasm,
//! the JS Parser package is available at the expected location.

use basegl_build_utilities::PathRef;
use basegl_build_utilities::absolute_path;
use basegl_build_utilities::targeting_wasm;

use std::fs::File;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_file;
use std::fs::write;
use std::io::Result;
use std::io::prelude::*;
use std::path::PathBuf;



// =========================
// == Hardcoded constants ==
// =========================

/// Where the crate expects to find file with compiled parser.
/// Path relative to the crate directory.
const PARSER_PATH: &str = "./pkg/scala-parser.js";

/// Commit from `enso` repository that will be used to obtain parser from.
const PARSER_COMMIT: &str = "417323deb2cbd26f1d61c914828eb0b1abdf28ff";

/// Magic code that needs to be prepended to ScalaJS generated parser due to:
/// https://github.com/scala-js/scala-js/issues/3677/
const PARSER_PREAMBLE: &str = "var __ScalaJSEnv = { global: window };";

/// Obtains a URL where this parser version can be downloaded.
pub fn download_url(version:&ParserVersion) -> reqwest::Url {
    let url_string = format!(
        "https://packages.luna-lang.org/parser-js/nightly/{}/scala-parser.js",
        version.commit);
    let invalid_url_msg = format!("{} is an invalid URL.", url_string);
    reqwest::Url::parse(&url_string).expect(&invalid_url_msg)
}



// ===============
// == Utilities ==
// ===============

/// Downloads file from given URL into the target file. The file contents will
/// be prepended with the preamble.
async fn download_file
(url:impl reqwest::IntoUrl, target:impl PathRef, preamble:&[u8]) {
    let invalid_url = format!("Invalid url given.");
    let url         = url.into_url().expect(&invalid_url);

    let get_error      = format!("Failed to get response from {}.",    url);
    let download_error = format!("Failed to download contents of {}.", url);
    let open_error     = format!("Failed to open {}.", target.as_ref().display());
    let write_error    = format!("Failed to write {}.",target.as_ref().display());
    let flush_error    = format!("Failed to flush {}.",target.as_ref().display());

    let response = reqwest::get(url).await.expect(&get_error);
    let bytes    = response.bytes().await.expect(&download_error);
    let mut file = File::create(&target).expect(&open_error);
    file.write_all(preamble).expect(&write_error);
    file.write_all(&bytes).expect(&write_error);
    file.flush().expect(&flush_error);
}



// ===================
// == ParserVersion ==
// ===================

/// Parser version described as commit hash from `enso` repository.
#[derive(Clone,Debug,PartialEq)]
pub struct ParserVersion{ pub commit:String }

impl ParserVersion {
    /// Create a version described by given commit hash.
    pub fn from_commit(commit:String) -> ParserVersion { ParserVersion{commit} }

    /// Write this version information to a file.
    pub fn store(&self, path:impl PathRef) -> Result<()> {
        write(path,&self.commit)
    }

    /// Load version information from a file.
    pub fn load(path:impl PathRef) -> Result<ParserVersion> {
        let commit = read_to_string(path)?;
        Ok(ParserVersion {commit})
    }

    /// The JS parser version required for this crate.
    pub fn required() -> ParserVersion {
        ParserVersion { commit: PARSER_COMMIT.into() }
    }
}



// ========================
// == Downloading parser ==
// ========================

/// Stores information which parser version should be provided where.
struct ParserProvider {
    /// Required parser version.
    version      : ParserVersion,
    /// The path where JS file needs to be provided.
    parser_path  : PathBuf,
    /// The path used to store version of the parser JS file.
    version_file : PathBuf,
}

impl ParserProvider {
    /// Creates a provider that obtains given parser version to a given path.
    pub fn new(version:ParserVersion, parser_path:impl PathRef) -> ParserProvider {
        let parser_path = PathBuf::from(parser_path.as_ref());
        let version_file = parser_path.with_extension("version");
        ParserProvider {version,parser_path,version_file}
    }

    /// Functions checks if parser in required version is already present.
    pub fn already_done(&self) -> bool {
        let parser_exists = match std::fs::metadata(&self.parser_path) {
            Ok(metadata) => metadata.is_file(),
            Err(_)       => false,
        };
        let has_correct_version = {
            let parser_local_version = ParserVersion::load(&self.version_file);
            parser_local_version.contains(&self.version)
        };
        parser_exists && has_correct_version
    }

    /// Ensures that target's parent directory exists.
    pub fn prepare_target_location(&self) {
        if let Some(parent_directory) = self.parser_path.parent() {
            let create_dir_error = format!(
                "Failed to create directory: {}.",
                parent_directory.display());
            create_dir_all(parent_directory).expect(&create_dir_error);
        }
    }

    /// Downloads JS parser and store its version for future reuse.
    pub async fn run(&self) {
        self.prepare_target_location();
        if !self.already_done() {
            remove_file(&self.version_file).ok();
            let url = download_url(&self.version);
            download_file(url, &self.parser_path, PARSER_PREAMBLE.as_bytes()).await;
            match self.version.store(&self.version_file) {
                Ok(_) => {}
                Err(e) => {
                    remove_file(&self.version_file).ok();
                    panic!("Failed to store parser's version info: {}.", e)
                }
            }
        }
    }
}



// ==========
// == main ==
// ==========

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    if targeting_wasm() {
        let required_version = ParserVersion::required();
        let parser_path      = absolute_path(PARSER_PATH)?;
        let provider = ParserProvider::new(required_version,&parser_path);
        provider.run().await;
    }
    Ok(())
}