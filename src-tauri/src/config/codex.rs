use std::{cmp::Ordering, fs, path::Path};

use serde_json::{json, Value};
use toml_edit::{value, DocumentMut, Item, Table};

use crate::{
    config::{backup, paths, FileWriteResult},
    error::{AppError, AppResult},
};

const MODEL_CATALOG_TEMPLATE: &str = include_str!("model_catalog_template.json");

pub fn write_codex_config(
    home: &Path,
    api_key: &str,
    openai_models: &[String],
) -> AppResult<Vec<FileWriteResult>> {
    let config_path = paths::codex_config_path(home);
    let auth_path = paths::codex_auth_path(home);
    let model_catalog_path = paths::codex_model_catalog_path(home);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let normalized_models = normalize_model_list(openai_models);

    let config_created = !config_path.exists();
    let auth_created = !auth_path.exists();
    let model_catalog_created = !model_catalog_path.exists();
    let config_backup = backup::backup_if_exists(&config_path)?;
    let auth_backup = backup::backup_if_exists(&auth_path)?;

    let mut doc = if config_path.exists() {
        fs::read_to_string(&config_path)?
            .parse::<DocumentMut>()
            .map_err(|err| AppError::Toml(err.to_string()))?
    } else {
        DocumentMut::new()
    };

    doc["model_provider"] = value("OpenAI");
    doc["model"] = value("gpt-5.4");
    doc["review_model"] = value("gpt-5.4");
    doc["model_reasoning_effort"] = value("xhigh");
    doc["disable_response_storage"] = value(true);
    doc["network_access"] = value("enabled");
    doc["windows_wsl_setup_acknowledged"] = value(true);
    doc["model_context_window"] = value(1_000_000);
    doc["model_auto_compact_token_limit"] = value(900_000);
    if normalized_models.is_empty() {
        doc.as_table_mut().remove("model_catalog_json");
    } else {
        doc["model_catalog_json"] = value(model_catalog_path.display().to_string());
    }

    if !doc.as_table().contains_key("model_providers") {
        doc["model_providers"] = Item::Table(Table::new());
    }
    if !doc["model_providers"].is_table() {
        doc["model_providers"] = Item::Table(Table::new());
    }
    doc["model_providers"]["OpenAI"] = Item::Table(Table::new());
    doc["model_providers"]["OpenAI"]["name"] = value("OpenAI");
    doc["model_providers"]["OpenAI"]["base_url"] = value("https://www.gigacoder.org/api");
    doc["model_providers"]["OpenAI"]["wire_api"] = value("responses");
    doc["model_providers"]["OpenAI"]["requires_openai_auth"] = value(true);
    fs::write(&config_path, doc.to_string())?;

    let mut auth = if auth_path.exists() {
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&auth_path)?)?
    } else {
        json!({})
    };
    if !auth.is_object() {
        auth = json!({});
    }
    auth.as_object_mut()
        .expect("auth object")
        .insert("OPENAI_API_KEY".to_string(), json!(api_key));
    fs::write(&auth_path, serde_json::to_string_pretty(&auth)?)?;

    let mut results = vec![
        FileWriteResult {
            path: config_path.display().to_string(),
            backup_path: config_backup.map(|item| item.display().to_string()),
            created: config_created,
        },
        FileWriteResult {
            path: auth_path.display().to_string(),
            backup_path: auth_backup.map(|item| item.display().to_string()),
            created: auth_created,
        },
    ];

    if !normalized_models.is_empty() {
        let model_catalog_backup = backup::backup_if_exists(&model_catalog_path)?;
        let catalog = build_custom_model_catalog(&normalized_models);
        fs::write(&model_catalog_path, serde_json::to_string_pretty(&catalog)?)?;
        results.push(FileWriteResult {
            path: model_catalog_path.display().to_string(),
            backup_path: model_catalog_backup.map(|item| item.display().to_string()),
            created: model_catalog_created,
        });
    }

    Ok(results)
}

fn normalize_model_list(models: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for model in models {
        let trimmed = model.trim();
        if trimmed.is_empty()
            || !is_codex_text_model(trimmed)
            || normalized
                .iter()
                .any(|item: &String| item.eq_ignore_ascii_case(trimmed))
        {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    sort_model_names_desc(&mut normalized);
    normalized
}

fn build_custom_model_catalog(models: &[String]) -> Value {
    json!({
        "models": models.iter().map(|model| custom_model_entry(model)).collect::<Vec<_>>()
    })
}

fn custom_model_entry(model: &str) -> Value {
    let mut entry = template_model_entry();
    let object = entry.as_object_mut().expect("template model entry object");
    object.insert("slug".to_string(), json!(model));
    object.insert("display_name".to_string(), json!(model));
    object.insert(
        "description".to_string(),
        json!("GigaCoder OpenAI-compatible model."),
    );
    object.insert("web_search_tool_type".to_string(), json!("text"));
    object.insert("max_context_window".to_string(), json!(1_000_000));
    object.insert("availability_nux".to_string(), Value::Null);
    object.insert("available_in_plans".to_string(), json!([]));
    object.insert("service_tiers".to_string(), json!([]));
    object.insert("additional_speed_tiers".to_string(), json!([]));
    entry
}

fn template_model_entry() -> Value {
    let catalog: Value =
        serde_json::from_str(MODEL_CATALOG_TEMPLATE).expect("model catalog template json");
    catalog["models"]
        .as_array()
        .and_then(|models| models.first())
        .cloned()
        .expect("model catalog template first model")
}

fn is_codex_text_model(model: &str) -> bool {
    !model.trim().to_ascii_lowercase().starts_with("gpt-image")
}

fn sort_model_names_desc(models: &mut [String]) {
    models.sort_by(|left, right| compare_model_names_desc(left, right));
}

fn compare_model_names_desc(left: &str, right: &str) -> Ordering {
    let left = left.to_ascii_lowercase();
    let right = right.to_ascii_lowercase();
    let mut left_index = 0;
    let mut right_index = 0;

    loop {
        if left_index >= left.len() && right_index >= right.len() {
            return Ordering::Equal;
        }
        if left_index >= left.len() {
            return Ordering::Less;
        }
        if right_index >= right.len() {
            return Ordering::Greater;
        }

        let left_is_digit = left.as_bytes()[left_index].is_ascii_digit();
        let right_is_digit = right.as_bytes()[right_index].is_ascii_digit();

        if left_is_digit && right_is_digit {
            let (left_segment, next_left) = digit_segment(&left, left_index);
            let (right_segment, next_right) = digit_segment(&right, right_index);
            let ordering = compare_digit_segments_desc(left_segment, right_segment);
            if ordering != Ordering::Equal {
                return ordering;
            }
            left_index = next_left;
            right_index = next_right;
            continue;
        }

        let (left_segment, next_left) = text_segment(&left, left_index);
        let (right_segment, next_right) = text_segment(&right, right_index);
        let ordering = left_segment.cmp(right_segment);
        if ordering != Ordering::Equal {
            return ordering;
        }
        left_index = next_left;
        right_index = next_right;
    }
}

fn digit_segment(value: &str, start: usize) -> (&str, usize) {
    let end = value[start..]
        .find(|character: char| !character.is_ascii_digit())
        .map(|offset| start + offset)
        .unwrap_or(value.len());
    (&value[start..end], end)
}

fn text_segment(value: &str, start: usize) -> (&str, usize) {
    let end = value[start..]
        .find(|character: char| character.is_ascii_digit())
        .map(|offset| start + offset)
        .unwrap_or(value.len());
    (&value[start..end], end)
}

fn compare_digit_segments_desc(left: &str, right: &str) -> Ordering {
    let left = left.trim_start_matches('0');
    let right = right.trim_start_matches('0');
    match left.len().cmp(&right.len()) {
        Ordering::Equal => right.cmp(left),
        other => other.reverse(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;
    use tempfile::tempdir;
    use toml_edit::DocumentMut;

    use super::*;

    #[test]
    fn creates_codex_config_and_auth() {
        let home = tempdir().expect("tempdir");

        let results = write_codex_config(home.path(), "gc_test_key", &[]).expect("write codex");

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|item| item.created));
        let config_path = home.path().join(".codex").join("config.toml");
        let auth_path = home.path().join(".codex").join("auth.json");
        let config = fs::read_to_string(config_path)
            .expect("read config")
            .parse::<DocumentMut>()
            .expect("toml");
        assert_eq!(config["model_provider"].as_str(), Some("OpenAI"));
        assert_eq!(
            config["model_providers"]["OpenAI"]["base_url"].as_str(),
            Some("https://www.gigacoder.org/api")
        );
        assert_eq!(
            config["model_providers"]["OpenAI"]["wire_api"].as_str(),
            Some("responses")
        );
        assert!(config.as_table().get("model_catalog_json").is_none());
        let auth: Value =
            serde_json::from_str(&fs::read_to_string(auth_path).expect("read auth")).expect("json");
        assert_eq!(auth["OPENAI_API_KEY"], "gc_test_key");
    }

    #[test]
    fn preserves_existing_codex_unknown_fields_and_creates_backups() {
        let home = tempdir().expect("tempdir");
        let codex_dir = home.path().join(".codex");
        fs::create_dir_all(&codex_dir).expect("mkdir");
        let config_path = codex_dir.join("config.toml");
        let auth_path = codex_dir.join("auth.json");
        fs::write(&config_path, "custom_flag = \"keep\"\n[other]\nvalue = 7\n")
            .expect("seed config");
        fs::write(
            &auth_path,
            r#"{"OTHER_TOKEN":"keep","OPENAI_API_KEY":"old"}"#,
        )
        .expect("seed auth");

        let results = write_codex_config(home.path(), "gc_new_key", &[]).expect("write codex");

        assert!(results.iter().all(|item| !item.created));
        assert!(results.iter().all(|item| item.backup_path.is_some()));
        for item in &results {
            assert!(fs::metadata(item.backup_path.as_ref().expect("backup")).is_ok());
        }
        let config = fs::read_to_string(config_path)
            .expect("read config")
            .parse::<DocumentMut>()
            .expect("toml");
        assert_eq!(config["custom_flag"].as_str(), Some("keep"));
        assert_eq!(config["other"]["value"].as_integer(), Some(7));
        assert_eq!(config["model"].as_str(), Some("gpt-5.4"));
        let auth: Value =
            serde_json::from_str(&fs::read_to_string(auth_path).expect("read auth")).expect("json");
        assert_eq!(auth["OTHER_TOKEN"], "keep");
        assert_eq!(auth["OPENAI_API_KEY"], "gc_new_key");
    }

    #[test]
    fn writes_custom_model_catalog_when_openai_models_are_available() {
        let home = tempdir().expect("tempdir");

        let results = write_codex_config(
            home.path(),
            "gc_test_key",
            &[
                "gpt-5.3-codex".to_string(),
                "gpt-5.4".to_string(),
                "gpt-5.4-mini".to_string(),
                "gpt-image-2".to_string(),
                "gpt-5.5".to_string(),
                " ".to_string(),
                "GPT-5.4".to_string(),
            ],
        )
        .expect("write codex");

        assert_eq!(results.len(), 3);
        let config_path = home.path().join(".codex").join("config.toml");
        let catalog_path = home.path().join(".codex").join("custom_models.json");
        let config = fs::read_to_string(config_path)
            .expect("read config")
            .parse::<DocumentMut>()
            .expect("toml");
        assert_eq!(
            config["model_catalog_json"].as_str(),
            Some(catalog_path.display().to_string().as_str())
        );

        let catalog: Value =
            serde_json::from_str(&fs::read_to_string(catalog_path).expect("read catalog"))
                .expect("json");
        let models = catalog["models"].as_array().expect("models array");
        assert_eq!(models.len(), 4);
        assert_eq!(models[0]["slug"], "gpt-5.5");
        assert_eq!(models[1]["slug"], "gpt-5.4");
        assert_eq!(models[2]["slug"], "gpt-5.4-mini");
        assert_eq!(models[3]["slug"], "gpt-5.3-codex");
        assert_eq!(models[0]["max_context_window"], 1_000_000);
        assert!(models[0]["base_instructions"].is_string());
        assert!(models[0]["model_messages"].is_object());
    }

    #[test]
    fn removes_model_catalog_config_when_openai_models_are_empty() {
        let home = tempdir().expect("tempdir");
        let codex_dir = home.path().join(".codex");
        fs::create_dir_all(&codex_dir).expect("mkdir");
        let config_path = codex_dir.join("config.toml");
        fs::write(
            &config_path,
            "model_catalog_json = \"/tmp/custom_models.json\"\n",
        )
        .expect("seed config");

        write_codex_config(home.path(), "gc_test_key", &[]).expect("write codex");

        let config = fs::read_to_string(config_path)
            .expect("read config")
            .parse::<DocumentMut>()
            .expect("toml");
        assert!(config.as_table().get("model_catalog_json").is_none());
    }
}
