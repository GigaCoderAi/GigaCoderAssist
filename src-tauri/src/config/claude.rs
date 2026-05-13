use std::{fs, path::Path};

use serde_json::{json, Map, Value};

use crate::{
    config::{backup, paths, FileWriteResult},
    error::AppResult,
};

pub fn write_claude_settings(home: &Path, api_key: &str) -> AppResult<FileWriteResult> {
    let path = paths::claude_settings_path(home);
    let created = !path.exists();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let backup_path = backup::backup_if_exists(&path)?;

    let mut root = if path.exists() {
        serde_json::from_str::<Value>(&fs::read_to_string(&path)?)?
    } else {
        json!({})
    };
    if !root.is_object() {
        root = json!({});
    }

    let object = root.as_object_mut().expect("root object");
    let env = object
        .entry("env")
        .or_insert_with(|| Value::Object(Map::new()));
    if !env.is_object() {
        *env = Value::Object(Map::new());
    }
    let env_object = env.as_object_mut().expect("env object");
    env_object.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        json!("https://www.gigacoder.org/api"),
    );
    env_object.insert("ANTHROPIC_AUTH_TOKEN".to_string(), json!(api_key));
    env_object.insert(
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
        json!("1"),
    );
    env_object.insert("CLAUDE_CODE_ATTRIBUTION_HEADER".to_string(), json!("0"));

    fs::write(&path, serde_json::to_string_pretty(&root)?)?;
    Ok(FileWriteResult {
        path: path.display().to_string(),
        backup_path: backup_path.map(|item| item.display().to_string()),
        created,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn creates_claude_settings_with_required_env() {
        let home = tempdir().expect("tempdir");

        let result = write_claude_settings(home.path(), "gc_test_key").expect("write settings");

        assert!(result.created);
        assert!(result.backup_path.is_none());
        let settings_path = home.path().join(".claude").join("settings.json");
        let settings: Value =
            serde_json::from_str(&fs::read_to_string(settings_path).expect("read settings"))
                .expect("json");
        assert_eq!(
            settings["env"]["ANTHROPIC_BASE_URL"],
            "https://www.gigacoder.org/api"
        );
        assert_eq!(settings["env"]["ANTHROPIC_AUTH_TOKEN"], "gc_test_key");
        assert_eq!(
            settings["env"]["CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"],
            "1"
        );
        assert_eq!(settings["env"]["CLAUDE_CODE_ATTRIBUTION_HEADER"], "0");
    }

    #[test]
    fn preserves_existing_claude_settings_and_creates_backup() {
        let home = tempdir().expect("tempdir");
        let settings_path = home.path().join(".claude").join("settings.json");
        fs::create_dir_all(settings_path.parent().expect("parent")).expect("mkdir");
        fs::write(
            &settings_path,
            r#"{"theme":"dark","env":{"EXISTING_FLAG":"yes","ANTHROPIC_AUTH_TOKEN":"old"}}"#,
        )
        .expect("seed");

        let result = write_claude_settings(home.path(), "gc_new_key").expect("write settings");

        assert!(!result.created);
        assert!(result.backup_path.is_some());
        let backup_path = result.backup_path.expect("backup");
        assert!(fs::metadata(backup_path).is_ok());
        let settings: Value =
            serde_json::from_str(&fs::read_to_string(settings_path).expect("read settings"))
                .expect("json");
        assert_eq!(settings["theme"], "dark");
        assert_eq!(settings["env"]["EXISTING_FLAG"], "yes");
        assert_eq!(settings["env"]["ANTHROPIC_AUTH_TOKEN"], "gc_new_key");
    }
}
