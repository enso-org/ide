use std::{fs, path};

/// Download the release package from github
pub fn github_download(
    project_url     : &str,
    version         : &str,
    filename        : &str,
    destination_dir : &path::Path
) {
    let url = format!(
        "{project}/releases/download/{version}/{filename}",
        project  = project_url,
        version  = version,
        filename = filename
    );

    let destination_file = path::Path::new(destination_dir)
        .join(filename);

    if destination_file.exists() {
        fs::remove_file(&destination_file).unwrap();
    }

    download_lp::download(
        url.as_str(),
        destination_dir.to_str().unwrap()
    ).unwrap();
}
