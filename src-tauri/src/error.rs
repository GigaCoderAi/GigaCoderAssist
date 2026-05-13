use std::io;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("无法确定用户主目录")]
    MissingHomeDir,
    #[error("文件读写失败: {0}")]
    Io(#[from] io::Error),
    #[error("JSON 处理失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML 处理失败: {0}")]
    Toml(String),
    #[error("{0}")]
    Http(String),
    #[error("GigaCoder 接口错误: {0}")]
    Api(String),
    #[error("INVALID_CREDENTIALS")]
    InvalidCredentials,
    #[error("配置目标不能为空")]
    EmptyTargets,
    #[error("API Key 不能为空")]
    EmptyApiKey,
}

pub type AppResult<T> = Result<T, AppError>;
