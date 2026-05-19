use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct LanguagePack {
    pub code: String,
    pub display_name: String,
    pub file_name: String,
    pub path: PathBuf,
    pub size: u64,
    pub sha256: String,
    pub modified_at: String,
    pub readable: bool,
    pub writable: bool,
}

// TODO: implement
pub fn scan_string_tables(_resource_path: &std::path::Path) -> Result<Vec<LanguagePack>, String> {
    Err("not implemented".into())
}
