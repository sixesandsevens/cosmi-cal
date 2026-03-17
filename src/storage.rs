// SPDX-License-Identifier: MPL-2.0

use crate::model::AppData;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

fn data_path() -> Result<PathBuf, String> {
    let proj = ProjectDirs::from("dev", "sixesandsevens", "cosmi-cal")
        .ok_or_else(|| "Could not determine config directory".to_string())?;
    let dir = proj.config_dir();
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    Ok(dir.join("data.json"))
}

pub fn save_data(data: &AppData) -> Result<(), String> {
    let path = data_path()?;
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_data() -> Result<AppData, String> {
    let path = data_path()?;
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}
