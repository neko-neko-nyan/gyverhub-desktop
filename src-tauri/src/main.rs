// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Context, WindowUrl, utils::{assets::EmbeddedAssets, config::AppUrl}, api::path::app_data_dir};
use url::Url;
use std::{fs, fs::File, path::Path, io};

const VERSION_URL: &str = "https://raw.githubusercontent.com/GyverLibs/GyverHub-web/main/version.txt";
const UPDATE_URL: &str = "https://raw.githubusercontent.com/GyverLibs/GyverHub-web/main/app/index.html";

fn get_online_version() -> reqwest::Result<String> {
    Ok(reqwest::blocking::get(VERSION_URL)?.text()?)
}

fn do_update(online_version: &str, version_path: &Path, gh_path: &Path) -> Result<(), String> {
    let mut resp = reqwest::blocking::get(UPDATE_URL).map_err(|e| e.to_string())?;
    let mut f = File::create(gh_path).map_err(|e| e.to_string())?;
    io::copy(&mut resp, &mut f).map_err(|e| e.to_string())?;
    fs::write(version_path, online_version).map_err(|e| e.to_string())?;
    Ok(())
}

fn check_updates(version_path: &Path, gh_path: &Path) -> bool {
    match get_online_version() {
        Ok(online_version) => {
            let current_version = fs::read_to_string(version_path).unwrap_or("".to_owned());
            if online_version != current_version {
                let result = do_update(&online_version, version_path, gh_path);
                match result {
                    Ok(_) => {}
                    Err(msg) => {
                        eprintln!("Warning: update failed!");
                        eprintln!("{}", msg);
                    }
                }
            }
            true
        }
        Err(err) => {
            eprintln!("Warning: update failed!");
            eprintln!("{}", err);

            if gh_path.exists() {
                true

            } else {
                eprintln!("Internet connection is required for first run!");
                false
            }
        }
    }
}

fn target_arch() -> &'static str {
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

fn target_os() -> &'static str {
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

fn main() {
    let mut ctx: Context<EmbeddedAssets> = tauri::generate_context!();
    let cfg: &mut tauri::Config = ctx.config_mut();

    let path = app_data_dir(&cfg).expect("Failed to find data dir!");

    let mut gh_path = path.clone();
    gh_path.push("gyverhub.html");

    let mut version_path = path.clone();
    version_path.push("version.txt");

    let _ = fs::create_dir(&path);  // do not check errors

    if check_updates(&version_path, &gh_path) {
        let url = Url::from_file_path(&gh_path).expect("Failed to build local URL!");
        cfg.tauri.windows[0].url = WindowUrl::External(url.clone());
        cfg.build.dev_path = AppUrl::Url(WindowUrl::External(url.clone()));
        cfg.build.dist_dir = AppUrl::Url(WindowUrl::External(url));
    }

    let desktop_version = cfg.package.version.clone().unwrap();

    tauri::Builder::default()
        .on_page_load(move |win, _payload| {
            let code = format!("window.GyverHubDesktop={{version:'{}',arch:'{}',os:'{}',debug:{}}};", desktop_version, target_arch(), target_os(), if cfg!(dev) {"true"} else {"false"});
            let _ = win.eval(&code);

            #[cfg(not(dev))]
            let _ = win.eval(
                "document.addEventListener('contextmenu',e=>(e.preventDefault(),!1),{capture:!0}),document.addEventListener('selectstart',e=>(e.preventDefault(),!1),{capture:!0});"
            );
        })
        .run(ctx)
        .expect("error while running tauri application");
}
