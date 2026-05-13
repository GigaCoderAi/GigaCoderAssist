use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

pub fn home_dir() -> AppResult<PathBuf> {
    dirs::home_dir().ok_or(AppError::MissingHomeDir)
}

pub fn claude_settings_path(home: &Path) -> PathBuf {
    home.join(".claude").join("settings.json")
}

pub fn codex_config_path(home: &Path) -> PathBuf {
    home.join(".codex").join("config.toml")
}

pub fn codex_auth_path(home: &Path) -> PathBuf {
    home.join(".codex").join("auth.json")
}

pub fn codex_model_catalog_path(home: &Path) -> PathBuf {
    home.join(".codex").join("custom_models.json")
}
