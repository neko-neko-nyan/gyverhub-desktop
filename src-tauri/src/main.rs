// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, fs::{File, remove_dir_all, remove_file}, io, path::Path};
use tauri::{
    api::{file::read_binary, path::app_local_data_dir, cli::Matches},
    http::ResponseBuilder,
    utils::assets::EmbeddedAssets,
    Context, Manager
};

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

fn has_cli_arg(matches: &Matches, flag: &str, default: bool) -> tauri::Result<bool> {
    match matches.args.get(flag) {
        Some(value) => {
            Ok(value.value.as_bool().unwrap())
        }
        None => Ok(default)
    }
}

fn print_error(e: io::Error) {
    println!("{}", e);
}

fn main() {
    static mut PAGE_DATA: Vec<u8> = vec![];

    let mut ctx: Context<EmbeddedAssets> = tauri::generate_context!();
    let cfg: &mut tauri::Config = ctx.config_mut();

    let path = app_local_data_dir(&cfg).expect("Failed to find data dir!");
    let gh_path = path.join("gyverhub.html");
    let version_path = path.join("version.txt");

    let app = tauri::Builder::default()
        .setup(move |app| {
            let matches = app.get_cli_matches()?;

            let use_builtin = has_cli_arg(&matches, "builtin", false)?;
            let keep_version = has_cli_arg(&matches, "keep-version", use_builtin)?;

            let clean_data = has_cli_arg(&matches, "clean", false)?;
            let force_update = has_cli_arg(&matches, "force-update", false)?;

            if clean_data {
                app.path_resolver().app_cache_dir().map_or(Ok(()), remove_dir_all).err().map(print_error);
                app.path_resolver().app_config_dir().map_or(Ok(()), remove_dir_all).err().map(print_error);
                #[cfg(target_os = "macos")]
                app.path_resolver().app_data_dir().map_or(Ok(()), remove_dir_all).err().map(print_error);

            } else if force_update {
                remove_file(version_path.clone()).err().map(print_error);
                remove_file(gh_path.clone()).err().map(print_error);
            }

            let use_fs = if use_builtin {
                true
            } else if keep_version {
                gh_path.exists()
            } else {
                let _ = fs::create_dir(&path); // do not check errors
                check_updates(&version_path, &gh_path)
            };

            let data = if use_fs {
                read_binary(&gh_path)?
            } else {
                app.asset_resolver().get(String::default()).unwrap().bytes
            };

            unsafe {
                PAGE_DATA = data;
            };

            Ok(())
        })
        .on_page_load(move |win, _payload| {
            let cfg = win.app_handle().config();
            let version = cfg.package.version.as_ref().unwrap();

            let code: String = format!("window.GyverHubDesktop={{version:'{}',arch:'{}',os:'{}',debug:{}}};",
                version, target_arch(), target_os(), if cfg!(dev) {"true"} else {"false"});
            let _ = win.eval(&code);

            #[cfg(not(dev))]
            let _ = win.eval(
                "document.addEventListener('contextmenu',e=>(e.preventDefault(),!1),{capture:!0}),document.addEventListener('selectstart',e=>(e.preventDefault(),!1),{capture:!0});"
            );
        })
        .register_uri_scheme_protocol("app",  |_, req| {
            if req.uri() == "app://localhost/" {
                ResponseBuilder::new().mimetype("text/html").body(unsafe { PAGE_DATA.clone() })
            } else {
                ResponseBuilder::new().status(404).body(Vec::new())
            }
        })
        .build(ctx).expect("error while building tauri application");

    
    app.run(|_, _| {});
}
