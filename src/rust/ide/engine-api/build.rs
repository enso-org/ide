use ensogl_build_utilities::absolute_path;

use std::fs;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use zip_extensions::read::ZipArchiveExtensions;



// =========================
// == Hardcoded constants ==
// =========================


/// The path where downloaded artefacts will be stored.
const ZIP_PATH   : &str = "./pkg/fbs-schema.zip";
/// The path where downloaded artefacts will be unziped.
const UNZIP_PATH : &str = "./pkg/fbs-upload/fbs-schema/";

/// Commit from `enso` repository that will be used to obtain artefacts from.
const COMMIT: &str = "25494bbb4315d7d6d625424280dfdee0e49dd045";

/// Obtains a URL where the artefacts can be downloaded.
pub fn flatbuffers_url() -> reqwest::Url {
    let url = format!("https://packages.luna-lang.org/fbs-schema/nightly/25494bbb4315d7d6d625424280dfdee0e49dd045/fbs-schema.zip");
    let err = format!("{} is an invalid URL.",url);
    reqwest::Url::parse(&url).expect(&err)
}

pub fn flatc(file:PathBuf) {
    let file_str = file.to_str().expect("Invalid UTF8 file path");
    let result   = Command::new("flatc")
        .args(&["--rust", "-o", "./src/generated", file_str])
        .output()
        .expect("Command `flatc` failed to execute");
    if !result.status.success() {
        panic!("Command `flatc` returned error exit code.")
    }
}

// ===================================
// == Download Engine Api Artefacts ==
// ===================================

/// Struct for downloading engine artefacts.
struct ApiProvider {
    /// The path where downloaded artefacts will be stored.
    zip_path: PathBuf,
    /// The path where downloaded artefacts will be unziped.
    unzip_path: PathBuf,
}

impl ApiProvider {
    /// Creates a provider that can download engine artefacts.
    pub fn new() -> ApiProvider {
        let zip_path   = PathBuf::from(absolute_path(ZIP_PATH).expect("Invalid path"));
        let unzip_path = PathBuf::from(absolute_path(UNZIP_PATH).expect("Invalid path"));
        ApiProvider{zip_path,unzip_path}
    }

    /// Downloads api artefacts into memory.
    pub async fn download(&self) -> bytes::Bytes {
        let url            = flatbuffers_url();
        let get_error      = format!("Failed to get response from {}",    &url);
        let download_error = format!("Failed to download contents of {}", &url);
        let response       = reqwest::get(url).await.expect(&get_error);
        response.bytes().await.expect(&download_error)
    }

    /// Saves unzipped artefacts into file.
    pub fn unzip(&self, artefacts:bytes::Bytes) {
        let display_path = self.zip_path.display();
        let open_error   = format!("Failed to open {}", display_path);
        let write_error  = format!("Failed to write {}",display_path);
        let flush_error  = format!("Failed to flush {}",display_path);
        let unzip_error  = format!("Failed to unzip {}",display_path);

        let mut file = File::create(&self.zip_path).expect(&open_error);
        file.write_all(&artefacts).expect(&write_error);
        file.flush().expect(&flush_error);

        let file = File::open(&self.zip_path).expect(&open_error);
        let root = self.zip_path.parent().expect("Unable to access parent directory");
        let mut archive = zip::ZipArchive::new(file).expect(&open_error);
        archive.extract(&root.to_path_buf()).expect(&unzip_error);
    }

    pub fn generate_files(&self) {
        for entry in fs::read_dir(&self.unzip_path).expect("Could not read content of dir") {
            let entry = entry.expect("Invalid content of dir");
            flatc(entry.path());
        }
    }

    /// Ensures that target's parent directory exists.
    pub fn prepare_target_location(&self) {
        let parent_directory = self.zip_path.parent().expect("Unable to access parent directory");
        let create_dir_error = format!(
            "Failed to create directory: {}.",
            parent_directory.display());
        create_dir_all(parent_directory).expect(&create_dir_error);
    }

    /// Places required artefacts in the target location.
    pub async fn run(&self) {
        self.prepare_target_location();
        let parent_directory = self.zip_path.parent().expect("Unable to access parent directory.");
        let fingerprint      = parent_directory.join("artefacts.fingerprint");
        let opt_version      = fs::read_to_string(&fingerprint);
        let changed          = match opt_version {
            Err(_)   => true,
            Ok(hash) => hash != COMMIT
        };
        if !changed {return}

        println!("cargo:warning=Engine API artefacts version changed. Rebuilding.");
        let artefacts = self.download().await;
        self.unzip(artefacts);
        self.generate_files();
        fs::write(&fingerprint,COMMIT).expect("Unable to write artefacts fingerprint.");
    }
}


// ==========
// == main ==
// ==========

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let provider = ApiProvider::new();
    provider.run().await;
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
