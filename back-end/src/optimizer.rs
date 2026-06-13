use crate::config::EmuType;
use crate::utils::io_error;
use include_dir::{include_dir, Dir};
use ini::Ini;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::config::{AdvancedOption, OptimizerConfig};
use crate::profile::UserProfile;

static BUNDLED_ARC_CONFIG: Dir = include_dir!("$CARGO_MANIFEST_DIR/bundled_data/arc_config");
static BUNDLED_ARC_MODS: Dir = include_dir!("$CARGO_MANIFEST_DIR/bundled_data/arc_mods");
static BUNDLED_ATMOSPHERE_APPS: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/bundled_data/atmosphere/contents");
static BUNDLED_SAVE_DATA: Dir = include_dir!("$CARGO_MANIFEST_DIR/bundled_data/save");
static BUNDLED_SSBU_SETTINGS: Dir = include_dir!("$CARGO_MANIFEST_DIR/bundled_data/ssbu_settings");

fn load_bundled_dir(bundled_dir: &Dir, target: PathBuf) -> io::Result<()> {
    log::info!("Creating directory path: {}", target.to_string_lossy());
    fs::create_dir_all(&target)?;
    for entry in bundled_dir.entries() {
        if let Some(dir) = entry.as_dir() {
            let relative_path =
                dir.path()
                    .strip_prefix(bundled_dir.path())
                    .ok()
                    .ok_or(io_error!(
                        NotFound,
                        "Unable to load bundled directory: {}",
                        dir.path().to_string_lossy()
                    ))?;
            load_bundled_dir(dir, target.join(relative_path))?;
        } else if let Some(file) = entry.as_file() {
            log::info!("Writing file: {}", file.path().to_string_lossy());
            fs::write(
                target.join(file.path().file_name().ok_or(io_error!(
                    NotFound,
                    "Unable to load bundled file: {}",
                    file.path().to_string_lossy()
                ))?),
                file.contents(),
            )?;
        }
    }
    Ok(())
}

pub fn optimize_settings(config: &OptimizerConfig, user_profile: &UserProfile) -> io::Result<()> {
    let config_folder = config.emu_filesystem.config_folder.as_ref();
    let emu_type = config.get_emulator_type();
    let emu_name = config.get_emulator_name();

    let ssbu_settings_path_src = BUNDLED_SSBU_SETTINGS
        .get_dir(emu_name.as_str())
        .or_else(|| BUNDLED_SSBU_SETTINGS.get_dir("common"))
        .ok_or(io_error!(
            NotFound,
            "Bundled custom SSBU settings not found"
        ))?;
    let ssbu_settings_path_dest = config_folder
        .ok_or(io_error!(NotFound, "Emulator config folder not found"))?
        .join("custom");
    load_bundled_dir(&ssbu_settings_path_src, ssbu_settings_path_dest)?;

    if emu_type == EmuType::Yuzu {
        let main_config_settings_path = config_folder
            .ok_or(io_error!(NotFound, "Emulator config folder not found"))?
            .join("qt-config.ini");
        let mut main_config = Ini::load_from_file_noescape(main_config_settings_path.as_path())
            .ok()
            .ok_or(io_error!(NotFound, "Unable to load main config"))?;
        let section = main_config
            .section_mut(Some("WebService"))
            .ok_or(io_error!(
                NotFound,
                "Unable to find WebService section in config"
            ))?;
        section.insert("enable_telemetry\\default", "false");
        section.insert("enable_telemetry", "false");
        section.insert("web_api_url\\default", "false");
        section.insert("web_api_url", "api.ynet-fun.xyz");
        section.insert(format!("{}_username\\default", emu_name), "false");
        section.insert(format!("{}_username", emu_name), user_profile.name.as_str());
        section.insert(format!("{}_token\\default", emu_name), "false");
        section.insert(
            format!("{}_token", emu_name),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        main_config
            .write_to_file_policy(
                main_config_settings_path.as_path(),
                ini::EscapePolicy::Nothing,
            )
            .ok()
            .ok_or(io_error!(NotFound, "Unable to save main config settings"))?;
    }
    Ok(())
}

pub fn optimize_mods(
    config: &OptimizerConfig,
    user_profile: &UserProfile,
    advanced_options: Vec<AdvancedOption>,
) -> io::Result<()> {
    let sdmc_path = config
        .emu_filesystem
        .sdmc_folder
        .as_ref()
        .ok_or(io_error!(NotFound, "Unable to find sdmc directory"))?;
    let atmosphere_apps_path = sdmc_path.join("atmosphere").join("contents");

    if advanced_options.contains(&AdvancedOption::CleanAtmosphere) && atmosphere_apps_path.is_dir()
    {
        log::info!("Removing atmosphere files...");
        for entry in BUNDLED_ATMOSPHERE_APPS.entries() {
            if let Some(dir) = entry.as_dir() {
                let app_id = dir
                    .path()
                    .file_name()
                    .ok_or(io_error!(NotFound, "Unable to get app id from bundled dir"))?
                    .to_str()
                    .ok_or(io_error!(
                        InvalidFilename,
                        "Unable to convert file name to str"
                    ))?;
                let app_dir = atmosphere_apps_path.join(app_id);
                log::info!("Removing atmosphere dir: {}", app_dir.to_string_lossy());
                fs::remove_dir_all(app_dir.as_path())?;
            }
        }
    }

    let arc_config_path = config.get_arc_config_folder(user_profile)?;
    let arc_mods_path = sdmc_path.join("ultimate");

    if advanced_options.contains(&AdvancedOption::CleanArc) && arc_mods_path.is_dir() {
        log::info!("Removing arcropolis files...");
        fs::remove_dir_all(arc_config_path.as_path())?;
        fs::remove_dir_all(arc_mods_path.as_path())?;
    }

    let arc_mods_path = arc_mods_path.join("mods");

    load_bundled_dir(&BUNDLED_ATMOSPHERE_APPS, atmosphere_apps_path)?;

    load_bundled_dir(&BUNDLED_ARC_CONFIG, arc_config_path)?;

    load_bundled_dir(&BUNDLED_ARC_MODS, arc_mods_path)?;

    Ok(())
}

pub fn optimize_save(config: &OptimizerConfig, user_profile: &UserProfile) -> io::Result<()> {
    let save_file_path = config.get_save_folder(user_profile)?;
    load_bundled_dir(&BUNDLED_SAVE_DATA, save_file_path)?;
    Ok(())
}
