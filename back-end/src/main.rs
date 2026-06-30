#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod optimizer;
mod profile;
mod utils;

use config::{CleanupOption, LocalPersistantData, Optimization, OptimizerConfig};
use profile::UserProfile;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use sysinfo::System;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{Target, TargetKind};

use crate::config::UserStatus;

struct AppState {
    app_handle: AppHandle,
    config: RwLock<OptimizerConfig>,
}

#[cfg(target_os = "linux")]
fn is_wayland_session() -> bool {
    matches!(
        std::env::var("XDG_SESSION_TYPE"),
        Ok(session_type) if session_type.eq_ignore_ascii_case("wayland")
    ) || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

#[cfg(target_os = "linux")]
fn wayland_workaround_disabled() -> bool {
    std::env::var_os("DISABLE_WAYLAND_WORKAROUND").is_some()
}

#[derive(Clone, Serialize)]
struct OptimizationStatusEvent {
    optimization: String,
    message: String,
    current: Option<usize>,
    total: Option<usize>,
}

impl AppState {
    fn check_web_engine_status(&self) {
        #[cfg(target_os = "windows")]
        if tauri::webview_version().is_err() {
            log::info!("Webview engine not found on system!");
            let app_handle = self.app_handle.clone();
            self.app_handle
                .dialog()
                .message("This app requires webview runtime to be installed!")
                .title("Webview Runtime Not Installed")
                .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCustom(
                    "Exit".into(),
                ))
                .show(move |_| app_handle.exit(0));
        }

        #[cfg(not(target_os = "windows"))]
        if tauri::webview_version().is_err() {
            log::info!("Webkit not found on system! Prompting warning message...");
            let app_handle = self.app_handle.clone();
            self.app_handle
                .dialog()
                .message("This app requires webkit to be installed!")
                .title("Webkit Not Installed")
                .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCustom(
                    "Exit".into(),
                ))
                .show(move |_| app_handle.exit(0));
        }
    }

    fn check_emu_not_running(&self) {
        let emu_name = self.read_config().get_emulator_name();
        let process_name = if cfg!(windows) {
            format!("{}.exe", emu_name)
        } else {
            emu_name.clone()
        };

        let mut system = System::new_all();
        if system
            .processes_by_exact_name(&process_name)
            .peekable()
            .peek()
            .is_some()
        {
            log::info!(
                "Detected at least one {} instance running. Prompting warning message...",
                emu_name
            );
            let app_handle = self.app_handle.clone();
            self.app_handle.dialog()
                .message(format!("At least one {} instance is detected running on your system. The optimizer works best if {} is closed. Close all {} instances?", emu_name, emu_name, emu_name))
                .title(format!("Close {} Instances", emu_name))
                .kind(tauri_plugin_dialog::MessageDialogKind::Warning)
                .buttons(tauri_plugin_dialog::MessageDialogButtons::YesNo)
                .show(move |terminate_emu| {
                    if terminate_emu {
                        system.refresh_all();
                        for process in system.processes_by_exact_name(&process_name) {
                            log::info!("Killing {} instance: {} ({})", emu_name, process.name(), process.pid());
                            process.kill();
                        }
                        app_handle.restart();
                    }
                });
        }
    }

    fn set_title(&self) {
        self.app_handle.get_webview_window("main").and_then(|w| {
            let app_version = env!("CARGO_PKG_VERSION");
            match w.set_title(format!("SSBU Emulator Optimizer v{}", app_version).as_str()) {
                Ok(_) => Some(w),
                Err(_) => None,
            }
        });
    }

    pub fn read_config(&self) -> RwLockReadGuard<'_, OptimizerConfig> {
        let config = self
            .config
            .read()
            .expect("Unable to acquire read lock on config");
        config
    }

    pub fn write_config(&self) -> RwLockWriteGuard<'_, OptimizerConfig> {
        let config = self
            .config
            .write()
            .expect("Unable to acquire read lock on config");
        config
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    if is_wayland_session() && !wayland_workaround_disabled() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::default()
                .clear_targets()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("ssbu-emu-optimizer".into()),
                    }),
                ])
                .level(log::LevelFilter::Debug)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .build(),
        )
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Unable to get app data directory");
            std::fs::create_dir_all(app_data_dir.as_path())
                .expect("Unable to create app data directory");
            let loaded_config = OptimizerConfig::load(app.app_handle().path(), None);
            app.manage(AppState {
                app_handle: app.app_handle().clone(),
                config: RwLock::new(loaded_config),
            });
            let state: tauri::State<AppState> = app.state();
            state.set_title();
            state.check_web_engine_status();
            state.check_emu_not_running();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let state: tauri::State<AppState> = window.state();
                let config = state.read_config();
                log::info!("Saving local data: {:#?}", config.local_data);
                config.local_data.save(window.app_handle().path());
            }
        })
        .invoke_handler(tauri::generate_handler![
            select_emu_data_folder,
            update_selected_user,
            apply_optimization,
            generate_sdcard_folder,
            query_mod_manifest,
            query_logs,
            open_logs_folder,
            query_app_version,
            query_app_build_uid,
            get_user_status,
            query_local_persistant_data,
            query_config,
            restart_app,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}

#[tauri::command]
async fn apply_optimization(
    app_handle: tauri::AppHandle,
    user_profile: UserProfile,
    optimization: Optimization,
    cleanup_options: Vec<CleanupOption>,
) -> Result<(), String> {
    let state: tauri::State<AppState> = app_handle.state();
    let config_snapshot = state.read_config().clone();
    log::info!(
        "Applying Optimization for user {}: {}",
        user_profile.name,
        optimization
    );

    let optimization_task = optimization.clone();
    let user_profile_task = user_profile.clone();
    let app_handle_task = app_handle.clone();
    let optimization_result =
        tauri::async_runtime::spawn_blocking(move || match optimization_task {
            Optimization::Settings => {
                optimizer::optimize_settings(&config_snapshot, &user_profile_task)
            }
            Optimization::Mods => {
                let emit_status = |message: &str, current: usize, total: usize| {
                    let _ = app_handle_task.emit(
                        "optimization-status",
                        OptimizationStatusEvent {
                            optimization: "Mods".to_string(),
                            message: message.to_string(),
                            current: Some(current),
                            total: Some(total),
                        },
                    );
                };
                optimizer::optimize_mods(
                    &config_snapshot,
                    &user_profile_task,
                    cleanup_options,
                    Some(&emit_status),
                )
            }
            Optimization::Save => optimizer::optimize_save(&config_snapshot, &user_profile_task),
        })
        .await
        .map_err(|err| format!("Optimization task failed: {}", err))?;

    if let Err(err) = optimization_result {
        log::error!("Error applying optimization: {}", err);
        return Err(err.to_string());
    }

    let mut config = state.write_config();
    let local_data = &mut config.local_data;
    match (
        optimization,
        local_data.user_statuses.get_mut(&user_profile),
    ) {
        (Optimization::Settings, Some(u)) => {
            u.settings_optimized = true;
        }
        (Optimization::Mods, Some(u)) => {
            u.mods_optimized = true;
        }
        (Optimization::Save, Some(u)) => {
            u.save_optimized = true;
        }
        (o, None) => {
            local_data.user_statuses.insert(
                user_profile,
                UserStatus {
                    settings_optimized: o == Optimization::Settings,
                    mods_optimized: o == Optimization::Mods,
                    save_optimized: o == Optimization::Save,
                },
            );
        }
    }
    config.local_data.save(app_handle.path());
    Ok(())
}

#[tauri::command]
async fn generate_sdcard_folder(
    app_handle: tauri::AppHandle,
    cleanup_options: Vec<CleanupOption>,
) -> Result<String, String> {
    let state: tauri::State<AppState> = app_handle.state();
    let default_directory = state
        .read_config()
        .local_data
        .emu_folder
        .clone()
        .or_else(|| app_handle.path().data_dir().ok())
        .ok_or("Unable to find default output directory".to_string())?;

    let app_handle_task = app_handle.clone();
    let emit_status = move |message: &str, current: usize, total: usize| {
        let _ = app_handle_task.emit(
            "optimization-status",
            OptimizationStatusEvent {
                optimization: "GenerateSdCard".to_string(),
                message: message.to_string(),
                current: Some(current),
                total: Some(total),
            },
        );
    };
    emit_status("Select location...", 0, 0);

    let dialog_result = app_handle
        .dialog()
        .file()
        .set_title("Select parent directory")
        .set_directory(default_directory)
        .blocking_pick_folder();

    let parent_dir = dialog_result
        .and_then(|value| value.into_path().ok())
        .ok_or("No output directory selected".to_string())?;

    let output_dir = parent_dir.join("generated-sdcard-root");

    let output_dir_task = output_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        optimizer::generate_sdcard_folder(
            output_dir_task.as_path(),
            cleanup_options,
            Some(&emit_status),
        )
        .map(|output_path| output_path.to_string_lossy().to_string())
    })
    .await
    .map_err(|err| format!("SD card generation task failed: {}", err))?
    .map_err(|err| err.to_string())
}

// should be called by the front-end only once, and then cached to avoid cloning too much
#[tauri::command]
fn query_config(state: tauri::State<AppState>) -> OptimizerConfig {
    state.read_config().clone()
}

// should be called by the front-end only once, and then cached to avoid cloning too much
#[tauri::command]
fn query_local_persistant_data(state: tauri::State<AppState>) -> LocalPersistantData {
    state.read_config().local_data.clone()
}

#[tauri::command]
async fn query_mod_manifest() -> Result<Vec<optimizer::ManifestModEntry>, String> {
    tauri::async_runtime::spawn_blocking(optimizer::list_manifest_mods)
        .await
        .map_err(|err| format!("Manifest query task failed: {}", err))?
        .map_err(|err| err.to_string())
}

fn preferred_log_file(logs_dir: &Path) -> Result<PathBuf, String> {
    let app_log = logs_dir.join("app.log");
    if app_log.is_file() {
        return Ok(app_log);
    }

    let mut candidates = fs::read_dir(logs_dir)
        .map_err(|err| format!("Unable to read logs directory: {}", err))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }

            let file_name = path.file_name()?.to_string_lossy().to_string();
            let is_log = file_name.ends_with(".log");
            if !is_log {
                return None;
            }

            let modified = entry
                .metadata()
                .ok()
                .and_then(|metadata| metadata.modified().ok());
            Some((path, modified))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Err("No log files found".to_string());
    }

    candidates.sort_by_key(|(_, modified)| *modified);
    let (path, _) = candidates.pop().ok_or("No log files found".to_string())?;
    Ok(path)
}

#[tauri::command]
async fn query_logs(app_handle: tauri::AppHandle) -> Result<String, String> {
    let logs_dir = app_handle
        .path()
        .app_log_dir()
        .map_err(|err| format!("Unable to resolve logs directory: {}", err))?;

    tauri::async_runtime::spawn_blocking(move || {
        let log_file = preferred_log_file(logs_dir.as_path())?;
        fs::read_to_string(log_file.as_path())
            .map_err(|err| format!("Unable to read log file: {}", err))
    })
    .await
    .map_err(|err| format!("Log query task failed: {}", err))?
}

#[tauri::command]
fn open_logs_folder(app_handle: tauri::AppHandle) -> Result<(), String> {
    let logs_dir = app_handle
        .path()
        .app_log_dir()
        .map_err(|err| format!("Unable to resolve logs directory: {}", err))?;

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(logs_dir.as_os_str())
            .spawn()
            .map_err(|err| format!("Unable to open logs directory: {}", err))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(logs_dir.as_os_str())
            .spawn()
            .map_err(|err| format!("Unable to open logs directory: {}", err))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(logs_dir.as_os_str())
            .spawn()
            .map_err(|err| format!("Unable to open logs directory: {}", err))?;
    }

    Ok(())
}

#[tauri::command]
fn query_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn query_app_build_uid() -> String {
    env!("APP_BUILD_UID").to_string()
}

#[tauri::command]
async fn select_emu_data_folder(app_handle: tauri::AppHandle) -> Result<OptimizerConfig, String> {
    let state: tauri::State<AppState> = app_handle.state();
    let config = state.read_config();
    let emu_folder = config.local_data.emu_folder.as_ref();
    let default_directory = app_handle
        .path()
        .data_dir()
        .expect("Unable to find data directory");
    let dialog_directory = emu_folder.unwrap_or(&default_directory);
    let dialog_result = app_handle
        .dialog()
        .file()
        .set_title("Select emulator data folder")
        .set_directory(dialog_directory)
        .blocking_pick_folder();
    drop(config);
    if let Some(f) = dialog_result {
        let folder = f
            .into_path()
            .expect("Unable to read selection as folder path");
        let new_config = OptimizerConfig::load(app_handle.path(), Some(folder));
        if new_config.local_data.emu_folder.is_none() {
            return Err(String::from("Incorrect emulator data folder specified"));
        }
        new_config.local_data.save(app_handle.path());
        let mut config = state.write_config();
        *config = new_config.clone();
        drop(config);
        state.check_emu_not_running();
        return Ok(new_config);
    }
    Err(String::from("No emulator data folder specified"))
}

#[tauri::command]
fn update_selected_user(state: tauri::State<AppState>, user_profile: Option<UserProfile>) {
    state.write_config().local_data.selected_user_profile = user_profile;
}

#[tauri::command]
fn get_user_status(state: tauri::State<AppState>, user_profile: UserProfile) -> UserStatus {
    if let Some(status) = state
        .read_config()
        .local_data
        .user_statuses
        .get(&user_profile)
    {
        return status.clone();
    }
    UserStatus::default()
}

#[tauri::command]
fn restart_app(app_handle: tauri::AppHandle) {
    app_handle.restart();
}
