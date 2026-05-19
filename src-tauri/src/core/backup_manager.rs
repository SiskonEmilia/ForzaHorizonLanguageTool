use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub tool_version: String,
    pub game: String,
    pub channel: String,
    pub game_root: String,
    pub resource_directory: String,
    pub voice_language: String,
    pub text_language: String,
    pub target_file: String,
    pub source_file: String,
    pub created_at: String,
    pub files: Vec<BackupFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFileEntry {
    pub path: String,
    pub original_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub id: String,
    pub game: String,
    pub voice_language: String,
    pub text_language: String,
    pub created_at: String,
    pub path: PathBuf,
    pub valid: bool,
}

// TODO: implement
pub fn create_backup(
    _game_id: &str,
    _voice_lang: &str,
    _text_lang: &str,
    _target_file: &std::path::Path,
) -> Result<PathBuf, String> {
    Err("not implemented".into())
}

// TODO: implement
pub fn list_backups(_game_id: &str) -> Result<Vec<BackupInfo>, String> {
    Err("not implemented".into())
}

// TODO: implement
pub fn get_backup_dir() -> Result<PathBuf, String> {
    Err("not implemented".into())
}
