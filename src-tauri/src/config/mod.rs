pub mod backup;
pub mod claude;
pub mod codex;
pub mod paths;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyConfigRequest {
    pub api_key: String,
    pub configure_claude: bool,
    pub configure_codex: bool,
    #[serde(default)]
    pub openai_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigPreview {
    pub claude_settings_path: Option<String>,
    pub codex_config_path: Option<String>,
    pub codex_auth_path: Option<String>,
    pub codex_model_catalog_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FileWriteResult {
    pub path: String,
    pub backup_path: Option<String>,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyConfigResponse {
    pub files: Vec<FileWriteResult>,
}

pub fn preview_configuration(
    configure_claude: bool,
    configure_codex: bool,
    openai_models: Vec<String>,
) -> AppResult<ConfigPreview> {
    if !configure_claude && !configure_codex {
        return Err(AppError::EmptyTargets);
    }

    let home = paths::home_dir()?;
    let claude = paths::claude_settings_path(&home);
    let codex_config = paths::codex_config_path(&home);
    let codex_auth = paths::codex_auth_path(&home);
    let codex_catalog = paths::codex_model_catalog_path(&home);
    let has_openai_models = openai_models.iter().any(|model| !model.trim().is_empty());

    Ok(ConfigPreview {
        claude_settings_path: configure_claude.then(|| claude.display().to_string()),
        codex_config_path: configure_codex.then(|| codex_config.display().to_string()),
        codex_auth_path: configure_codex.then(|| codex_auth.display().to_string()),
        codex_model_catalog_path: (configure_codex && has_openai_models)
            .then(|| codex_catalog.display().to_string()),
    })
}

pub fn apply_configuration(request: ApplyConfigRequest) -> AppResult<ApplyConfigResponse> {
    if request.api_key.trim().is_empty() {
        return Err(AppError::EmptyApiKey);
    }
    if !request.configure_claude && !request.configure_codex {
        return Err(AppError::EmptyTargets);
    }

    let home = paths::home_dir()?;
    let mut files = Vec::new();
    if request.configure_claude {
        files.push(claude::write_claude_settings(
            &home,
            request.api_key.trim(),
        )?);
    }
    if request.configure_codex {
        files.extend(codex::write_codex_config(
            &home,
            request.api_key.trim(),
            &request.openai_models,
        )?);
    }

    Ok(ApplyConfigResponse { files })
}
