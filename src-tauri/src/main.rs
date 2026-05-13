mod config;
mod error;
mod gigacoder;

use config::{
    apply_configuration, preview_configuration, ApplyConfigRequest, ApplyConfigResponse,
    ConfigPreview,
};
use gigacoder::{login_and_fetch_keys, LoginAndFetchKeysResponse};

#[tauri::command]
async fn login_and_fetch_keys_command(
    email: String,
    password: String,
) -> Result<LoginAndFetchKeysResponse, String> {
    login_and_fetch_keys(email, password)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn preview_configuration_command(
    configure_claude: bool,
    configure_codex: bool,
    openai_models: Vec<String>,
) -> Result<ConfigPreview, String> {
    preview_configuration(configure_claude, configure_codex, openai_models)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn apply_configuration_command(request: ApplyConfigRequest) -> Result<ApplyConfigResponse, String> {
    apply_configuration(request).map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            login_and_fetch_keys_command,
            preview_configuration_command,
            apply_configuration_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
