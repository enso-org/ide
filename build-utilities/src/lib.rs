#![feature(option_unwrap_none)]

use std::{fs, path};
use std::io::ErrorKind;


/// A structure describing a concrete release package on github
pub struct GithubRelease<'a,'b,'c> {
    pub project_url : &'a str,
    pub version     : &'b str,
    pub filename    : &'c str,
}

impl GithubRelease<'_, '_, '_> {
    /// Download the release package from github
    ///
    /// The project_url should be a project's main page on github.
    pub fn download(&self, destination_dir:&path::Path) {
        let url = format!(
            "{project}/releases/download/{version}/{filename}",
            project  = self.project_url,
            version  = self.version,
            filename = self.filename);

        let destination_dir_str = destination_dir.to_str().unwrap();
        let destination_file    = destination_dir.join(self.filename);

        Self::remove_old_file(&destination_file);

        download_lp::download(url.as_str(),destination_dir_str).unwrap();
    }

    fn remove_old_file(file:&path::Path) {
        let result      = fs::remove_file(&file);
        let error       = result.err();
        let fatal_error = error.filter(|err| err.kind() != ErrorKind::NotFound);
        fatal_error.unwrap_none();
    }
}
