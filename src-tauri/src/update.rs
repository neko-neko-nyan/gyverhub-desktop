use std::{io, fs, path::Path};
use serde::Serialize;

const VERSION_URL: &str = "https://raw.githubusercontent.com/GyverLibs/GyverHub-web/main/version.txt";
const UPDATE_URL: &str = "https://raw.githubusercontent.com/GyverLibs/GyverHub-web/main/app/index.html";

#[derive(Serialize)]
pub(crate) enum UpdateStatus {
    Disabled,       // Disabled by --keep-version
    Updated,        // Update installed
    Skipped,        // Already updated
    NetworkError,   // Download failed
    IOError,        // Write failed, try --force-update
}

pub(crate) fn update(version_path: &Path, gh_path: &Path) -> (UpdateStatus, Option<String>) {
    let online_version = match reqwest::blocking::get(VERSION_URL).and_then(|resp| resp.text()) {
        Ok(v) => v,
        Err(err) => return (UpdateStatus::NetworkError, Some(err.to_string())),
    }; 

    let current_version = fs::read_to_string(version_path);
    let must_update = current_version.map_or(true, |cv| cv != online_version);
    if ! must_update {
        return (UpdateStatus::Skipped, None);
    }

    let mut resp = match reqwest::blocking::get(UPDATE_URL) {
        Ok(resp) => resp,
        Err(err) => return (UpdateStatus::NetworkError, Some(err.to_string())),
    };
    
    let res = fs::File::create(gh_path).and_then(|mut f| {
        io::copy(&mut resp, &mut f)?;
        fs::write(version_path, online_version)
    });

    match res {
        Ok(_) => (UpdateStatus::Updated, None),
        Err(err) => (UpdateStatus::IOError, Some(err.to_string())),
    }
}
