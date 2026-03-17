// SPDX-License-Identifier: MPL-2.0

use crate::model::AppData;
use std::fs;
use std::path::PathBuf;

fn data_path() -> Result<PathBuf, String> {
    // Deliberately ignore XDG_CONFIG_HOME so the path is stable regardless of
    // which environment (terminal, desktop launcher, flatpak wrapper) the app
    // is started from.
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let dir = PathBuf::from(home).join(".config").join("cosmical");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("data.json"))
}

pub fn save_data(data: &AppData) -> Result<(), String> {
    let path = data_path()?;
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;

    // Write to a temp file in the same directory, then rename into place.
    // This makes the save atomic: a crash mid-write leaves the old file intact.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

pub fn load_data() -> Result<AppData, String> {
    let path = data_path()?;
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}
