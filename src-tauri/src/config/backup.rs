use std::{
    fs,
    path::{Path, PathBuf},
};

use time::{macros::format_description, OffsetDateTime};

use crate::error::AppResult;

pub fn backup_if_exists(path: &Path) -> AppResult<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    let timestamp = OffsetDateTime::now_utc()
        .format(format_description!(
            "[year][month][day]-[hour][minute][second]"
        ))
        .unwrap_or_else(|_| "timestamp".to_string());
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config");
    let backup_path = path.with_file_name(format!("{file_name}.gigacoder.{timestamp}.bak"));
    fs::copy(path, &backup_path)?;
    Ok(Some(backup_path))
}
