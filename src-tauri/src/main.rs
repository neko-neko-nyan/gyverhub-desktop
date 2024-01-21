// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod update;
mod utils;

use serde::Serialize;
use std::{cell::OnceCell, fs};
use tauri::{http::ResponseBuilder, Manager};

#[derive(Serialize)]
struct GyverHubDesktopConfig {
    version: String,
    arch: &'static str,
    os: &'static str,
    debug: bool,
    update_status: update::UpdateStatus,
    update_error: Option<String>,
    clean_data: bool,
}

fn main() {
    thread_local! {
        static INIT_SCRIPT: OnceCell<String> = OnceCell::new();
        static PAGE_DATA: OnceCell<Vec<u8>> = OnceCell::new();
    }

    tauri::Builder::default()
        .setup(move |app| {
            let cfg = app.config();
            let matches = app.get_cli_matches()?;

            if let Some(value) = matches.args.get("help") {
                print!("{}", value.value.as_str().unwrap());
                let _ = app.windows().get("main").unwrap().close();
                return Ok(());
            }

            if matches.args.contains_key("version") {
                println!("{}", cfg.package.version.as_ref().unwrap().clone());
                let _ = app.windows().get("main").unwrap().close();
                return Ok(());
            }

            let use_builtin = utils::has_cli_arg(&matches, "builtin", false);
            let keep_version = utils::has_cli_arg(&matches, "keep-version", use_builtin);

            let clean_data = utils::has_cli_arg(&matches, "clean", false);
            let force_update = utils::has_cli_arg(&matches, "force-update", false);

            let path = app
                .path_resolver()
                .app_local_data_dir()
                .expect("Failed to find data dir!");
            let gh_path = path.join("gyverhub.html");
            let version_path = path.join("version.txt");

            let mut config = GyverHubDesktopConfig {
                version: cfg.package.version.as_ref().unwrap().clone(),
                arch: utils::target_arch(),
                os: utils::target_os(),
                debug: cfg!(dev),
                update_status: update::UpdateStatus::Disabled,
                update_error: None,
                clean_data,
            };

            if force_update {
                utils::remove_file_report(&version_path);
                utils::remove_file_report(&gh_path);
            }

            if clean_data {
                if let Some(path) = app.path_resolver().app_cache_dir() {
                    utils::remove_dir_report(&path);
                }
                if let Some(path) = app.path_resolver().app_config_dir() {
                    utils::remove_dir_report(&path);
                }
                #[cfg(all(not(windows), target_os = "macos"))]
                if let Some(path) = app.path_resolver().app_data_dir() {
                    utils::remove_dir_report(&path);
                }
            }

            if !keep_version {
                utils::create_dir_report(&path);
                (config.update_status, config.update_error) =
                    update::update(&version_path, &gh_path);
            }

            let data = if use_builtin {
                None
            } else {
                fs::read(&gh_path).ok()
            };
            let data =
                data.unwrap_or_else(|| app.asset_resolver().get(String::default()).unwrap().bytes);

            PAGE_DATA.with(|value| value.set(data)).unwrap();

            let mut script = "window.GyverHubDesktop=".to_owned();
            script.push_str(&serde_json::to_string(&config).unwrap());
            INIT_SCRIPT.with(|value| value.set(script)).unwrap();

            // TODO remove when web api will do it
            app.windows().get("main").unwrap().show().unwrap();

            Ok(())
        })
        .on_page_load(move |win, _payload| {
            let code = INIT_SCRIPT.with(|value| value.get().unwrap().clone());
            let _ = win.eval(&code);

            // TODO remove when web api will do it
            #[cfg(not(dev))]
            let _ = win.eval(
                "document.addEventListener('contextmenu',e=>(e.preventDefault(),!1),{capture:!0}),document.addEventListener('selectstart',e=>(e.preventDefault(),!1),{capture:!0});"
            );
        })
        .register_uri_scheme_protocol("app", |_, req| {
            if req.uri() == "app://localhost/" {
                let data = PAGE_DATA.with(|value| value.get().unwrap().clone());
                ResponseBuilder::new().mimetype("text/html").body(data)
            } else {
                ResponseBuilder::new().status(404).body(Vec::new())
            }
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
