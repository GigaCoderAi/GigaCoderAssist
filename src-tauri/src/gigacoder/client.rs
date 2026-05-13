use std::{collections::HashSet, error::Error};

use reqwest::{Client, ClientBuilder, Error as ReqwestError, Proxy, StatusCode};
use serde_json::json;

use crate::{
    error::{AppError, AppResult},
    gigacoder::types::*,
};

const API_BASE: &str = "https://www.gigacoder.org/api/v1";

pub async fn login_and_fetch_keys(
    email: String,
    password: String,
) -> AppResult<LoginAndFetchKeysResponse> {
    let client = build_client()?;
    let login = external_login(&client, &email, &password).await?;
    let openai_catalog = public_openai_model_catalog(&client)
        .await
        .unwrap_or_default();
    let openai_models = openai_models_for_login(&login.platforms, &login.models, &openai_catalog);
    Ok(LoginAndFetchKeysResponse {
        user_email: login.user.email,
        keys: selectable_keys_for_login(login.api_key, login.api_keys),
        platforms: login.platforms,
        models: normalize_models(login.models),
        openai_models,
        fallback_used: false,
    })
}

fn build_client() -> AppResult<Client> {
    let mut builder = ClientBuilder::new();
    if let Some(proxy_url) = system_proxy_url() {
        let proxy = Proxy::all(&proxy_url).map_err(|err| {
            AppError::Http(format!("系统代理配置无效：{proxy_url}。原始错误：{err}"))
        })?;
        builder = builder.proxy(proxy);
    }
    builder.build().map_err(describe_http_error)
}

#[cfg(target_os = "macos")]
fn system_proxy_url() -> Option<String> {
    use std::process::Command;

    let output = Command::new("scutil").arg("--proxy").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let enabled = proxy_value(&text, "HTTPSEnable").or_else(|| proxy_value(&text, "HTTPEnable"));
    if enabled.as_deref() != Some("1") {
        return None;
    }
    let host = proxy_value(&text, "HTTPSProxy")
        .or_else(|| proxy_value(&text, "HTTPProxy"))
        .filter(|value| !value.trim().is_empty())?;
    let port = proxy_value(&text, "HTTPSPort").or_else(|| proxy_value(&text, "HTTPPort"))?;
    Some(format!("http://{}:{}", host.trim(), port.trim()))
}

#[cfg(not(target_os = "macos"))]
fn system_proxy_url() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
fn proxy_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let trimmed = line.trim();
        let (name, value) = trimmed.split_once(':')?;
        (name.trim() == key).then(|| value.trim().to_string())
    })
}

async fn external_login(
    client: &Client,
    email: &str,
    password: &str,
) -> AppResult<ExternalLoginResponse> {
    let response = client
        .post(format!("{API_BASE}/external/auth/login"))
        .json(&json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(describe_http_error)?;
    let status = response.status();
    if !status.is_success() {
        if matches!(
            status,
            StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            return Err(AppError::InvalidCredentials);
        }
        return Err(AppError::Api(format!(
            "获取默认 API Key 失败: HTTP {status}"
        )));
    }
    response
        .json::<ExternalLoginResponse>()
        .await
        .map_err(describe_http_error)
}

async fn public_openai_model_catalog(client: &Client) -> AppResult<Vec<String>> {
    let response = client
        .get(format!("{API_BASE}/model-pricings/public"))
        .send()
        .await
        .map_err(describe_http_error)?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::Api(format!("获取模型列表失败: HTTP {status}")));
    }
    let envelope = response
        .json::<V1Envelope<PublicModelPricingResponse>>()
        .await
        .map_err(describe_http_error)?;
    if envelope.code != 0 {
        return Err(AppError::Api(envelope.message));
    }
    Ok(envelope
        .data
        .platforms
        .into_iter()
        .find(|platform| platform.code.eq_ignore_ascii_case("openai"))
        .map(|platform| {
            platform
                .models
                .into_iter()
                .map(|model| model.model_name)
                .collect()
        })
        .unwrap_or_default())
}

pub fn selectable_keys(keys: Vec<ApiKeyPayload>) -> Vec<ApiKeySummary> {
    keys.into_iter()
        .filter(|key| key.status == "active")
        .filter_map(|key| {
            let raw_key = key.raw_key.unwrap_or_default();
            if raw_key.trim().is_empty() || key.has_full_key == Some(false) {
                return None;
            }
            Some(ApiKeySummary {
                id: key.id,
                name: key.name,
                masked_key: mask_key(&raw_key),
                raw_key,
                status: key.status,
                expires_at: key.expires_at,
            })
        })
        .collect()
}

fn selectable_keys_for_login(
    default_key: ApiKeyPayload,
    api_keys: Vec<ApiKeyPayload>,
) -> Vec<ApiKeySummary> {
    let keys = if api_keys.is_empty() {
        vec![default_key]
    } else {
        api_keys
    };
    selectable_keys(keys)
}

pub fn mask_key(raw_key: &str) -> String {
    let trimmed = raw_key.trim();
    if trimmed.len() <= 12 {
        return format!("{trimmed}...");
    }
    format!("{}...{}", &trimmed[..8], &trimmed[trimmed.len() - 4..])
}

pub fn openai_models_for_login(
    platforms: &[PlatformSummary],
    models: &[String],
    catalog_models: &[String],
) -> Vec<String> {
    let normalized = normalize_models(models.to_vec());
    if normalized.is_empty() {
        return Vec::new();
    }

    let has_openai_platform = platforms
        .iter()
        .any(|platform| platform.code.eq_ignore_ascii_case("openai"));
    let catalog: HashSet<String> = catalog_models
        .iter()
        .map(|model| model.trim().to_ascii_lowercase())
        .filter(|model| !model.is_empty())
        .collect();

    normalized
        .into_iter()
        .filter(|model| is_codex_text_model(model))
        .filter(|model| {
            let catalog_match =
                !catalog.is_empty() && catalog.contains(&model.to_ascii_lowercase());
            if has_openai_platform {
                catalog_match || looks_like_openai_model(model)
            } else {
                looks_like_openai_model(model)
            }
        })
        .collect()
}

fn normalize_models(models: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for model in models {
        let trimmed = model.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_ascii_lowercase();
        if seen.insert(key) {
            normalized.push(trimmed.to_string());
        }
    }
    normalized
}

fn looks_like_openai_model(model: &str) -> bool {
    let lower = model.trim().to_ascii_lowercase();
    lower.starts_with("gpt-")
        || lower.starts_with("o1")
        || lower.starts_with("o3")
        || lower.starts_with("o4")
}

fn is_codex_text_model(model: &str) -> bool {
    !model.trim().to_ascii_lowercase().starts_with("gpt-image")
}

fn describe_http_error(error: ReqwestError) -> AppError {
    let detail = error.to_string();
    let source_chain = error
        .source()
        .map(|source| {
            let mut parts = vec![source.to_string()];
            let mut current = source.source();
            while let Some(next) = current {
                parts.push(next.to_string());
                current = next.source();
            }
            parts.join(": ")
        })
        .unwrap_or_default();
    let combined = format!("{detail}: {source_chain}").to_lowercase();

    let message = if combined.contains("dns")
        || combined.contains("resolve")
        || combined.contains("name or service not known")
        || combined.contains("could not resolve")
        || combined.contains("nodename nor servname")
    {
        format!("无法解析 GigaCoder 域名，请检查 DNS 或网络连接。原始错误：{detail}")
    } else if error.is_timeout() || combined.contains("timed out") || combined.contains("timeout") {
        format!("连接 GigaCoder 超时，请检查网络后重试。原始错误：{detail}")
    } else if error.is_connect()
        || combined.contains("connection")
        || combined.contains("tcp connect")
    {
        format!("无法连接 GigaCoder 服务，请检查网络、防火墙或代理设置。原始错误：{detail}")
    } else if error.is_decode() {
        format!("GigaCoder 返回内容无法解析，请稍后重试或联系管理员。原始错误：{detail}")
    } else {
        format!("HTTP 请求失败。原始错误：{detail}")
    };

    AppError::Http(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(
        id: &str,
        status: &str,
        raw_key: Option<&str>,
        has_full_key: Option<bool>,
    ) -> ApiKeyPayload {
        ApiKeyPayload {
            id: id.to_string(),
            raw_key: raw_key.map(str::to_string),
            has_full_key,
            name: format!("key-{id}"),
            status: status.to_string(),
            expires_at: None,
        }
    }

    #[test]
    fn selectable_keys_only_keeps_active_keys_with_full_raw_key() {
        let keys = selectable_keys(vec![
            key("1", "active", Some("gc_1234567890abcdef"), Some(true)),
            key("2", "inactive", Some("gc_inactive"), Some(true)),
            key("3", "active", None, Some(false)),
            key("4", "expired", Some("gc_expired"), Some(true)),
        ]);

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, "1");
        assert_eq!(keys[0].raw_key, "gc_1234567890abcdef");
    }

    #[test]
    fn selectable_keys_for_login_prefers_api_key_list() {
        let keys = selectable_keys_for_login(
            key("default", "active", Some("gc_default"), Some(true)),
            vec![
                key("1", "active", Some("gc_1234567890abcdef"), Some(true)),
                key("2", "inactive", Some("gc_inactive"), Some(true)),
            ],
        );

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, "1");
    }

    #[test]
    fn mask_key_keeps_prefix_and_suffix() {
        assert_eq!(mask_key("gc_1234567890abcdef"), "gc_12345...cdef");
    }

    #[test]
    fn openai_models_use_catalog_with_name_detection_fallback() {
        let platforms = vec![PlatformSummary {
            code: "openai".to_string(),
            name: "OpenAI".to_string(),
        }];
        let models = vec![
            "claude-sonnet-4-6".to_string(),
            "gpt-5.3-codex".to_string(),
            "gpt-5.4".to_string(),
            "gpt-5.4-mini".to_string(),
            "gpt-image-2".to_string(),
            "gpt-5.5".to_string(),
        ];
        let catalog = vec!["gpt-5.4".to_string(), "gpt-image-2".to_string()];

        let openai = openai_models_for_login(&platforms, &models, &catalog);

        assert_eq!(
            openai,
            vec!["gpt-5.3-codex", "gpt-5.4", "gpt-5.4-mini", "gpt-5.5"]
        );
    }

    #[test]
    fn openai_models_preserve_login_model_order() {
        let platforms = vec![PlatformSummary {
            code: "openai".to_string(),
            name: "OpenAI".to_string(),
        }];
        let models = vec![
            "gpt-5.4-mini".to_string(),
            "gpt-5.5".to_string(),
            "gpt-5.3-codex".to_string(),
        ];

        let openai = openai_models_for_login(&platforms, &models, &[]);

        assert_eq!(openai, vec!["gpt-5.4-mini", "gpt-5.5", "gpt-5.3-codex"]);
    }

    #[test]
    fn openai_models_fallback_to_name_detection_when_catalog_is_empty() {
        let platforms = vec![PlatformSummary {
            code: "openai".to_string(),
            name: "OpenAI".to_string(),
        }];
        let models = vec![
            "claude-sonnet-4-6".to_string(),
            "gpt-5.4".to_string(),
            "o3".to_string(),
        ];

        let openai = openai_models_for_login(&platforms, &models, &[]);

        assert_eq!(openai, vec!["gpt-5.4", "o3"]);
    }

    #[test]
    fn openai_models_can_be_detected_when_platforms_are_missing() {
        let platforms = vec![PlatformSummary {
            code: "anthropic".to_string(),
            name: "Anthropic".to_string(),
        }];
        let models = vec!["gpt-5.4".to_string()];

        let openai = openai_models_for_login(&platforms, &models, &[]);

        assert_eq!(openai, vec!["gpt-5.4"]);
    }

    #[test]
    fn openai_models_are_empty_without_openai_like_model_names() {
        let platforms = vec![PlatformSummary {
            code: "anthropic".to_string(),
            name: "Anthropic".to_string(),
        }];
        let models = vec!["claude-sonnet-4-6".to_string()];

        let openai = openai_models_for_login(&platforms, &models, &[]);

        assert!(openai.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn proxy_value_reads_scutil_proxy_output() {
        let sample = r#"
<dictionary> {
  HTTPEnable : 1
  HTTPPort : 7897
  HTTPProxy : 127.0.0.1
}
"#;

        assert_eq!(proxy_value(sample, "HTTPEnable").as_deref(), Some("1"));
        assert_eq!(
            proxy_value(sample, "HTTPProxy").as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(proxy_value(sample, "HTTPPort").as_deref(), Some("7897"));
    }
}
