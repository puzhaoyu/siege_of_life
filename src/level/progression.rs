use std::path::PathBuf;

use directories::ProjectDirs;

use crate::level::data::SaveData;

fn get_save_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "siege-of-life", "Siege of Life")
        .map(|dirs| dirs.data_dir().join("progress.json"))
}

pub fn load_progress() -> SaveData {
    let path = match get_save_path() {
        Some(p) => p,
        None => return SaveData::default(),
    };

    if !path.exists() {
        return SaveData::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => SaveData::default(),
    }
}

pub fn save_progress(data: &SaveData) {
    let path = match get_save_path() {
        Some(p) => p,
        None => return,
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(&path, json);
    }
}
