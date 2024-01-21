use std::{io, fs, path::Path};
use tauri::api::cli::Matches;

pub(crate) fn target_arch() -> &'static str {
    if cfg!(target_arch = "x86") {
        "i686"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "arm") {
        "armv7"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    }
}

pub(crate) fn target_os() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "freebsd") {
        "freebsd"
    } else {
        "unknown"
    }
}

pub(crate) fn has_cli_arg(matches: &Matches, flag: &str, default: bool) -> bool {
    matches.args.get(flag).and_then(|value| value.value.as_bool()).unwrap_or(default)
}

pub(crate) fn remove_file_report(path: &Path) {
    match fs::remove_file(path) {
        Ok(_) => (),
        Err(err) => if err.kind() != io::ErrorKind::NotFound {
            print_error("removing file", err, path)
        }
    }
}

pub(crate) fn remove_dir_report(path: &Path) {
    match fs::remove_dir_all(path) {
        Ok(_) => (),
        Err(err) => if err.kind() != io::ErrorKind::NotFound {
            print_error("removing directory", err, path)
        }
    }
}


pub(crate) fn create_dir_report(path: &Path) {
    match fs::create_dir_all(path) {
        Ok(_) => (),
        Err(err) => if err.kind() != io::ErrorKind::AlreadyExists {
            print_error("creating directory", err, path)
        }
    }
}

fn print_error(action: &str, err: io::Error, path: &Path) {
    eprintln!("Error {action} {}: {err}", path.to_string_lossy());
}
