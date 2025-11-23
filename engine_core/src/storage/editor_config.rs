// editor/src/storage/editor_config.rs
use std::sync::RwLock;
use ron::from_str;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories_next::ProjectDirs;
use once_cell::sync::Lazy;
use std::fs;
use std::io::ErrorKind;

use crate::onscreen_log;

pub static CONFIG: Lazy<RwLock<EditorConfig>> = Lazy::new(|| RwLock::new(load_config()));

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EditorConfig {
    pub save_root: Option<PathBuf>,
}

/// Saves the editor config .ron file from the in memory config.
pub fn save_config() {
    let path = config_path();
    
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let config = CONFIG.read().expect("Failed to lock CONFIG for reading");
    
    let ron = to_string_pretty(&*config, PrettyConfig::default())
        .expect("Serialization failed.");

    let _ = fs::write(path, ron);
}

/// Gets the config save root. Returns `None` if the lock is poisoned
/// or if the field itself is `None`.
pub fn get_save_root() -> Option<PathBuf> {
    if let Err(e) = CONFIG.read() {
        onscreen_log!("Could not read config: {e}.");
        None
    } else {
        CONFIG.read().unwrap().save_root.clone()
    }
}

/// Returns a clone of the current in-memory config.
pub fn clone_config() -> Option<EditorConfig> {
    if let Err(e) = CONFIG.read() {
        onscreen_log!("Could not read save root: {e}.");
        None
    } else {
        Some(CONFIG.read().unwrap().clone())
    }
}

pub fn app_dir() -> PathBuf {
    let project_dir = ProjectDirs::from("com", "bishop", "engine")
        .expect("Could not locate platform config directory.");

    project_dir.config_dir().to_path_buf()
}

fn config_path() -> PathBuf {
    app_dir().join("editor_config.ron")
}

fn load_config() -> EditorConfig {
    let path = config_path();

    match fs::read_to_string(&path) {
        Ok(txt) =>  {from_str(&txt).unwrap_or_default()},
        Err(e) if e.kind() == ErrorKind::NotFound => EditorConfig::default(),
        Err(_) => EditorConfig::default(),
    }
}