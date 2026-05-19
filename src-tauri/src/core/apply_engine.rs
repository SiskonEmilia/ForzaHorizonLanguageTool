use std::fs;
use std::path::Path;

use serde::Serialize;

use super::backup_manager;
use super::game_detector::GameProfile;
use super::language_mapper::ApplyPlan;
use super::logger;
use super::resource_scanner;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub success: bool,
    pub message: String,
    pub backup_path: Option<String>,
    pub rolled_back: bool,
}

pub fn execute_apply(plan: &ApplyPlan, profile: &GameProfile) -> ApplyResult {
    let copy_op = match plan.operations.iter().find(|op| op.op_type == "copy_replace") {
        Some(op) => op,
        None => {
            return ApplyResult {
                success: false,
                message: "Apply plan has no copy_replace operation".into(),
                backup_path: None,
                rolled_back: false,
            };
        }
    };

    let backup_op = match plan.operations.iter().find(|op| op.op_type == "backup") {
        Some(op) => op,
        None => {
            return ApplyResult {
                success: false,
                message: "Apply plan has no backup operation".into(),
                backup_path: None,
                rolled_back: false,
            };
        }
    };

    let source_path = Path::new(&copy_op.from);
    let target_path = Path::new(&backup_op.from);

    if !source_path.is_file() {
        return ApplyResult {
            success: false,
            message: format!(
                "Source file does not exist: {}",
                source_path.display()
            ),
            backup_path: None,
            rolled_back: false,
        };
    }

    if !target_path.is_file() {
        return ApplyResult {
            success: false,
            message: format!(
                "Target file does not exist: {}",
                target_path.display()
            ),
            backup_path: None,
            rolled_back: false,
        };
    }

    let backup_root = match backup_manager::get_backup_dir() {
        Ok(dir) => dir,
        Err(e) => {
            return ApplyResult {
                success: false,
                message: format!("Failed to get backup directory: {}", e),
                backup_path: None,
                rolled_back: false,
            };
        }
    };

    let backup_path = match backup_manager::create_backup_to(
        &backup_root,
        &plan.game_id,
        &profile.channel,
        &profile.root_path,
        &profile.resource_path,
        &plan.voice_language,
        &plan.text_language,
        target_path,
        &plan.source_file,
    ) {
        Ok(path) => path,
        Err(e) => {
            return ApplyResult {
                success: false,
                message: format!("Failed to create backup: {}", e),
                backup_path: None,
                rolled_back: false,
            };
        }
    };

    let backup_path_str = backup_path.to_string_lossy().to_string();

    let temp_path = target_path.with_extension("tmp");

    if let Err(e) = fs::copy(source_path, &temp_path) {
        let _ = rollback_from_backup(&backup_path, target_path);
        return ApplyResult {
            success: false,
            message: format!("Failed to copy source to temp file: {}", e),
            backup_path: Some(backup_path_str),
            rolled_back: rollback_from_backup_check(&backup_path, target_path),
        };
    }

    let source_hash = match resource_scanner::compute_sha256(source_path) {
        Ok(h) => h,
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            return ApplyResult {
                success: false,
                message: format!("Failed to compute SHA-256 of source: {}", e),
                backup_path: Some(backup_path_str),
                rolled_back: false,
            };
        }
    };

    let temp_hash = match resource_scanner::compute_sha256(&temp_path) {
        Ok(h) => h,
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            return ApplyResult {
                success: false,
                message: format!("Failed to compute SHA-256 of temp file: {}", e),
                backup_path: Some(backup_path_str),
                rolled_back: false,
            };
        }
    };

    if source_hash != temp_hash {
        let _ = fs::remove_file(&temp_path);
        return ApplyResult {
            success: false,
            message: format!(
                "SHA-256 mismatch after copy: source={}, temp={}",
                source_hash, temp_hash
            ),
            backup_path: Some(backup_path_str),
            rolled_back: false,
        };
    }

    if let Err(e) = fs::rename(&temp_path, target_path) {
        let _ = fs::remove_file(&temp_path);
        let rolled_back = rollback_from_backup_check(&backup_path, target_path);
        return ApplyResult {
            success: false,
            message: format!("Failed to rename temp file to target: {}", e),
            backup_path: Some(backup_path_str),
            rolled_back,
        };
    }

    match resource_scanner::compute_sha256(target_path) {
        Ok(final_hash) if final_hash == source_hash => {}
        Ok(final_hash) => {
            let rolled_back = rollback_from_backup_check(&backup_path, target_path);
            return ApplyResult {
                success: false,
                message: format!(
                    "Final verification failed: expected {}, got {}",
                    source_hash, final_hash
                ),
                backup_path: Some(backup_path_str),
                rolled_back,
            };
        }
        Err(e) => {
            let rolled_back = rollback_from_backup_check(&backup_path, target_path);
            return ApplyResult {
                success: false,
                message: format!("Failed to verify final target: {}", e),
                backup_path: Some(backup_path_str),
                rolled_back,
            };
        }
    }

    let _ = logger::log_operation(
        &plan.game_id,
        "info",
        &format!(
            "Applied {} text to {} voice (backup: {})",
            plan.text_language, plan.voice_language, backup_path_str
        ),
    );

    ApplyResult {
        success: true,
        message: format!(
            "Successfully applied {} text language to {} voice language pack",
            plan.text_language, plan.voice_language
        ),
        backup_path: Some(backup_path_str),
        rolled_back: false,
    }
}

fn rollback_from_backup(backup_path: &Path, target_path: &Path) -> Result<(), String> {
    let manifest = backup_manager::read_manifest(backup_path)?;
    let file_entry = manifest
        .files
        .first()
        .ok_or_else(|| "Backup manifest has no file entries".to_string())?;
    let original_file = backup_path.join("original").join(&file_entry.path);
    fs::copy(&original_file, target_path)
        .map_err(|e| format!("Failed to restore from backup: {}", e))?;
    Ok(())
}

fn rollback_from_backup_check(backup_path: &Path, target_path: &Path) -> bool {
    rollback_from_backup(backup_path, target_path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_detector::GameId;
    use crate::core::language_mapper::{ApplyPlan, Operation};
    use std::fs;
    use std::path::PathBuf;

    fn make_temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("fh_apply_engine_test")
            .join(format!("{}_{}", label, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_profile(root: &Path, resource: &Path) -> GameProfile {
        GameProfile {
            game_id: GameId::Fh5,
            display_name: "Forza Horizon 5".into(),
            channel: "steam".into(),
            steam_app_id: "1551360".into(),
            root_path: root.to_path_buf(),
            resource_path: resource.to_path_buf(),
            executable_name: "ForzaHorizon5.exe".into(),
        }
    }

    #[test]
    fn test_execute_apply_success() {
        let dir = make_temp_dir("apply_success");
        let resource_dir = dir.join("resources");
        fs::create_dir_all(&resource_dir).unwrap();

        let source = resource_dir.join("CHS.zip");
        let target = resource_dir.join("EN.zip");
        fs::write(&source, b"chinese-text-data").unwrap();
        fs::write(&target, b"english-voice-data").unwrap();

        let plan = ApplyPlan {
            game_id: "fh5".into(),
            voice_language: "EN".into(),
            text_language: "CHS".into(),
            source_file: "CHS.zip".into(),
            target_file: "EN.zip".into(),
            operations: vec![
                Operation {
                    op_type: "backup".into(),
                    from: target.to_string_lossy().into_owned(),
                    to: "ignored".into(),
                    description: "Backup EN.zip".into(),
                },
                Operation {
                    op_type: "copy_replace".into(),
                    from: source.to_string_lossy().into_owned(),
                    to: target.to_string_lossy().into_owned(),
                    description: "Copy CHS.zip -> EN.zip".into(),
                },
            ],
        };

        let profile = make_profile(&dir, &resource_dir);
        let result = execute_apply(&plan, &profile);

        assert!(result.success, "Expected success, got: {}", result.message);
        assert!(result.backup_path.is_some());
        assert!(!result.rolled_back);

        let target_content = fs::read(&target).unwrap();
        assert_eq!(target_content, b"chinese-text-data");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_execute_apply_missing_source() {
        let dir = make_temp_dir("apply_missing_source");
        let resource_dir = dir.join("resources");
        fs::create_dir_all(&resource_dir).unwrap();

        let source = resource_dir.join("CHS.zip");
        let target = resource_dir.join("EN.zip");
        fs::write(&target, b"english-voice-data").unwrap();

        let plan = ApplyPlan {
            game_id: "fh5".into(),
            voice_language: "EN".into(),
            text_language: "CHS".into(),
            source_file: "CHS.zip".into(),
            target_file: "EN.zip".into(),
            operations: vec![
                Operation {
                    op_type: "backup".into(),
                    from: target.to_string_lossy().into_owned(),
                    to: "ignored".into(),
                    description: "Backup EN.zip".into(),
                },
                Operation {
                    op_type: "copy_replace".into(),
                    from: source.to_string_lossy().into_owned(),
                    to: target.to_string_lossy().into_owned(),
                    description: "Copy CHS.zip -> EN.zip".into(),
                },
            ],
        };

        let profile = make_profile(&dir, &resource_dir);
        let result = execute_apply(&plan, &profile);

        assert!(!result.success);
        assert!(
            result.message.contains("Source file does not exist"),
            "Expected missing source message, got: {}",
            result.message
        );
        assert!(result.backup_path.is_none());

        let _ = fs::remove_dir_all(&dir);
    }
}
