use std::fs;
use std::path::Path;

use serde::Serialize;

use super::backup_manager;
use super::logger;
use super::resource_scanner;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub success: bool,
    pub message: String,
}

pub fn execute_restore(backup_path: &Path) -> RestoreResult {
    let manifest = match backup_manager::read_manifest(backup_path) {
        Ok(m) => m,
        Err(e) => {
            return RestoreResult {
                success: false,
                message: format!("Failed to read backup manifest: {}", e),
            };
        }
    };

    let file_entry = match manifest.files.first() {
        Some(entry) => entry,
        None => {
            return RestoreResult {
                success: false,
                message: "Backup manifest contains no file entries".into(),
            };
        }
    };

    let original_file = backup_path.join("original").join(&file_entry.path);
    if !original_file.is_file() {
        return RestoreResult {
            success: false,
            message: format!(
                "Backup original file does not exist: {}",
                original_file.display()
            ),
        };
    }

    let backup_hash = match resource_scanner::compute_sha256(&original_file) {
        Ok(h) => h,
        Err(e) => {
            return RestoreResult {
                success: false,
                message: format!("Failed to compute SHA-256 of backup file: {}", e),
            };
        }
    };

    if backup_hash != file_entry.original_sha256 {
        return RestoreResult {
            success: false,
            message: format!(
                "Backup file integrity check failed: expected {}, got {}",
                file_entry.original_sha256, backup_hash
            ),
        };
    }

    let resource_dir = Path::new(&manifest.resource_directory);
    let target_path = resource_dir.join(&manifest.target_file);

    let temp_path = target_path.with_extension("tmp");

    if let Err(e) = fs::copy(&original_file, &temp_path) {
        return RestoreResult {
            success: false,
            message: format!("Failed to copy backup to temp file: {}", e),
        };
    }

    match resource_scanner::compute_sha256(&temp_path) {
        Ok(h) if h == file_entry.original_sha256 => {}
        Ok(h) => {
            let _ = fs::remove_file(&temp_path);
            return RestoreResult {
                success: false,
                message: format!(
                    "Temp file integrity check failed: expected {}, got {}",
                    file_entry.original_sha256, h
                ),
            };
        }
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            return RestoreResult {
                success: false,
                message: format!("Failed to verify temp file: {}", e),
            };
        }
    }

    if let Err(e) = fs::rename(&temp_path, &target_path) {
        let _ = fs::remove_file(&temp_path);
        return RestoreResult {
            success: false,
            message: format!("Failed to rename temp file to target: {}", e),
        };
    }

    match resource_scanner::compute_sha256(&target_path) {
        Ok(final_hash) if final_hash == file_entry.original_sha256 => {}
        Ok(final_hash) => {
            return RestoreResult {
                success: false,
                message: format!(
                    "Restored file verification failed: expected {}, got {}",
                    file_entry.original_sha256, final_hash
                ),
            };
        }
        Err(e) => {
            return RestoreResult {
                success: false,
                message: format!("Failed to verify restored file: {}", e),
            };
        }
    }

    if let (Some(mp), Some(orig_lang)) = (&manifest.manifest_path, &manifest.original_steam_language) {
        let mp_path = std::path::Path::new(mp);
        if mp_path.exists() {
            match super::steam_language::set_manifest_language(mp_path, orig_lang) {
                Ok(_) => { let _ = logger::log_operation(&manifest.game, "info", &format!("Restored Steam language to '{}'", orig_lang)); }
                Err(e) => { let _ = logger::log_operation(&manifest.game, "warn", &format!("Failed to restore Steam language: {}", e)); }
            }
        }
    }

    let _ = logger::log_operation(
        &manifest.game,
        "info",
        &format!(
            "Restored {} from backup ({})",
            manifest.target_file,
            backup_path.display()
        ),
    );

    RestoreResult {
        success: true,
        message: format!(
            "Successfully restored {} to original state",
            manifest.target_file
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::backup_manager::{BackupFileEntry, BackupManifest};
    use std::fs;
    use std::path::PathBuf;

    fn make_temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("fh_restore_engine_test")
            .join(format!("{}_{}", label, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_execute_restore_success() {
        let dir = make_temp_dir("restore_success");

        let resource_dir = dir.join("resources");
        fs::create_dir_all(&resource_dir).unwrap();

        let target_file = resource_dir.join("EN.zip");
        fs::write(&target_file, b"modified-content").unwrap();

        let original_content = b"original-english-data";
        let original_hash = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(original_content);
            format!("{:x}", hasher.finalize())
        };

        let backup_dir = dir.join("backup_001");
        let original_dir = backup_dir.join("original");
        fs::create_dir_all(&original_dir).unwrap();
        fs::write(original_dir.join("EN.zip"), original_content).unwrap();

        let manifest = BackupManifest {
            tool_version: "1.0.0".into(),
            game: "fh5".into(),
            channel: "steam".into(),
            game_root: dir.to_string_lossy().into_owned(),
            resource_directory: resource_dir.to_string_lossy().into_owned(),
            voice_language: "EN".into(),
            text_language: "CHS".into(),
            target_file: "EN.zip".into(),
            source_file: "CHS.zip".into(),
            created_at: "2025-01-01T00:00:00+00:00".into(),
            manifest_path: None,
            original_steam_language: None,
            files: vec![BackupFileEntry {
                path: "EN.zip".into(),
                original_sha256: original_hash,
            }],
        };

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(backup_dir.join("manifest.json"), manifest_json).unwrap();

        let result = execute_restore(&backup_dir);

        assert!(result.success, "Expected success, got: {}", result.message);

        let restored_content = fs::read(&target_file).unwrap();
        assert_eq!(restored_content, original_content);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_execute_restore_missing_manifest() {
        let dir = make_temp_dir("restore_no_manifest");
        let backup_dir = dir.join("backup_missing");
        fs::create_dir_all(&backup_dir).unwrap();

        let result = execute_restore(&backup_dir);

        assert!(!result.success);
        assert!(
            result.message.contains("Failed to read backup manifest"),
            "Expected manifest error, got: {}",
            result.message
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
