use flatc_rust;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use zip_extensions::read::ZipArchiveExtensions;



// =========================
// == Hardcoded constants ==
// =========================

/// The name of zip containing engine interface files.
const ZIP_NAME:&str = "fbs-schema.zip";

/// The directory structure inside downloaded engine interface folder.
const ZIP_CONTENT:&str = "fbs-upload/fbs-schema/";

/// Commit from `enso` repository that will be used to obtain artefacts from.
/// If you change this commit manually, you must have `flatc` installed to regenrate inteface files.
/// If you are contributor, you are obligated to also run `cargo build` before creating a commit
/// with such change!
/// Hint: You can install flatc with `conda -c conda-forge install flatbuffers=1.12.0`
const COMMIT:&str = "29190f83392e5da04172c36ea432c5410770da0f";

/// An URL pointing to engine interface files.
pub fn interface_description_url() -> reqwest::Url {
    let url = format!("https://packages.luna-lang.org/fbs-schema/nightly/{}/fbs-schema.zip",COMMIT);
    let err = format!("{} is an invalid URL.",url);
    reqwest::Url::parse(&url).expect(&err)
}



// ===================================
// == Download Engine Api Artefacts ==
// ===================================

/// Struct for downloading engine artefacts.
struct ApiProvider {
    /// The path where downloaded artefacts will be stored.
    out_dir: PathBuf,
}

impl ApiProvider {
    /// Creates a provider that can download engine artefacts.
    pub fn new() -> ApiProvider {
        let out_dir_str = env::var("OUT_DIR").expect("OUT_DIR isn't environment variable");
        ApiProvider{out_dir:out_dir_str.into()}
    }

    /// Downloads api artefacts into memory.
    pub async fn download(&self) -> bytes::Bytes {
        let url            = interface_description_url();
        let get_error      = format!("Failed to get response from {}",    &url);
        let download_error = format!("Failed to download contents of {}", &url);
        let response       = reqwest::get(url).await.expect(&get_error);
        response.bytes().await.expect(&download_error)
    }

    /// Saves unzipped artefacts into file.
    pub fn unzip(&self, artefacts:bytes::Bytes) {
        let zip_path     = self.out_dir.join(ZIP_NAME);
        let display_path = zip_path.display();
        let open_error   = format!("Failed to open {}", display_path);
        let write_error  = format!("Failed to write {}",display_path);
        let flush_error  = format!("Failed to flush {}",display_path);
        let unzip_error  = format!("Failed to unzip {}",display_path);

        let mut file = File::create(&zip_path).expect(&open_error);
        file.write_all(&artefacts).expect(&write_error);
        file.flush().expect(&flush_error);

        let file        = File::open(&zip_path).expect(&open_error);
        let mut archive = zip::ZipArchive::new(&file).expect(&open_error);
        archive.extract(&self.out_dir).expect(&unzip_error);
    }


    /// Generates rust files from flatbuffers schemas.
    pub fn generate_files(&self) {
        let fbs_dir = self.out_dir.join(ZIP_CONTENT);
        for entry in fs::read_dir(&fbs_dir).expect("Could not read content of dir") {
            let path = entry.expect("Invalid content of dir").path();
            let result = flatc_rust::run(flatc_rust::Args {
                inputs  : &[&path],
                out_dir : &PathBuf::from("./src"),
                ..Default::default()
            });
            if result.is_err() {
                println!("cargo:info=Engine API files were not regenerated because `flatc` isn't installed");
                break;
            }
        }
    }

    /// Places required artefacts in the target location.
    pub async fn run(&self) {
        let fingerprint = self.out_dir.join("egine.api.fingerprint");
        let unchanged   = match fs::read_to_string(&fingerprint) {
            Ok(commit) => commit == COMMIT,
            Err(_)     => false,
        };
        if unchanged {return}

        println!("cargo:info=Engine API artefacts version changed. Rebuilding.");
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
