use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ApiKeySummary {
    pub id: String,
    pub name: String,
    pub masked_key: String,
    pub raw_key: String,
    pub status: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginAndFetchKeysResponse {
    pub user_email: String,
    pub keys: Vec<ApiKeySummary>,
    pub platforms: Vec<PlatformSummary>,
    pub models: Vec<String>,
    pub openai_models: Vec<String>,
    pub fallback_used: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct V1Envelope<T> {
    pub code: i32,
    pub message: String,
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserPayload {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeyPayload {
    pub id: String,
    pub raw_key: Option<String>,
    pub has_full_key: Option<bool>,
    pub name: String,
    pub status: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalLoginResponse {
    pub user: UserPayload,
    pub api_key: ApiKeyPayload,
    #[serde(default)]
    pub platforms: Vec<PlatformSummary>,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlatformSummary {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublicModelPricingResponse {
    #[serde(default)]
    pub platforms: Vec<PublicModelPricingPlatform>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublicModelPricingPlatform {
    pub code: String,
    #[serde(default)]
    pub models: Vec<PublicModelPricingModel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublicModelPricingModel {
    pub model_name: String,
}
