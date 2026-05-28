use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::core::backup_manager;
use crate::core::resource_scanner;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigStatus {
    pub state: String,
    pub game_id: Option<String>,
    pub voice_language: Option<String>,
    pub text_language: Option<String>,
    pub last_applied: Option<String>,
}

#[tauri::command]
pub fn check_game_running(game_id: String) -> Result<bool, String> {
    let process_name = match game_id.as_str() {
        "fh5" => "ForzaHorizon5.exe",
        "fh6" => "forzahorizon6.exe",
        _ => return Err(format!("Unknown game_id: {game_id}")),
    };

    let output = match Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {process_name}"), "/NH"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Ok(false),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(process_name))
}

#[tauri::command]
pub fn get_status(game_id: String, resource_path: String) -> Result<ConfigStatus, String> {
    let backups = backup_manager::list_backups(&game_id)?;

    if backups.is_empty() {
        return Ok(ConfigStatus {
            state: "none".into(),
            game_id: None,
            voice_language: None,
            text_language: None,
            last_applied: None,
        });
    }

    let latest = &backups[0];
    let manifest = backup_manager::read_manifest(&latest.path)?;

    let resource_dir = Path::new(&resource_path);
    let target_path = resource_dir.join(&manifest.target_file);

    let current_hash = resource_scanner::compute_sha256(&target_path)
        .map_err(|e| format!("Failed to compute SHA-256 of current file: {}", e))?;

    let source_file_path = resource_dir.join(&manifest.source_file);
    let source_hash = resource_scanner::compute_sha256(&source_file_path).ok();

    let original_hash = manifest.files.first().map(|f| f.original_sha256.as_str());

    let state = classify_state(
        &current_hash,
        source_hash.as_deref(),
        original_hash,
        manifest.applied_sha256.as_deref(),
    );

    Ok(ConfigStatus {
        state: state.into(),
        game_id: Some(manifest.game),
        voice_language: Some(manifest.voice_language),
        text_language: Some(manifest.text_language),
        last_applied: Some(manifest.created_at),
    })
}

/// Classify the current state of the target (voice) file relative to the latest
/// backup, given the hashes of:
/// - `current`: the target file currently on disk (e.g. `EN.zip`)
/// - `source`: the latest source text pack on disk (e.g. `CHS.zip`), if readable
/// - `original`: the original target content recorded in the backup manifest
/// - `applied`: the content we wrote into the target at apply time
///   (= source text pack at apply time), if recorded
///
/// Priority:
/// 1. matches latest source text pack    -> `applied`  (override active & up to date)
/// 2. matches recorded original          -> `reverted` (back to the untouched voice pack)
/// 3. matches what we applied            -> `outdated` (override intact, but text pack updated)
/// 4. otherwise                          -> `modified`
///
/// Backups created before `applied_sha256` existed pass `applied = None`, so the
/// `outdated` branch never fires for them and behaviour falls back to `modified`.
fn classify_state(
    current: &str,
    source: Option<&str>,
    original: Option<&str>,
    applied: Option<&str>,
) -> &'static str {
    if source == Some(current) {
        "applied"
    } else if original == Some(current) {
        "reverted"
    } else if applied == Some(current) {
        "outdated"
    } else {
        "modified"
    }
}

#[cfg(test)]
mod tests {
    use super::classify_state;

    #[test]
    fn test_applied_when_target_matches_latest_source() {
        // EN.zip currently equals the latest CHS.zip -> override active & in sync.
        assert_eq!(
            classify_state("chs_v2", Some("chs_v2"), Some("en_orig"), Some("chs_v2")),
            "applied"
        );
    }

    #[test]
    fn test_reverted_when_target_matches_original() {
        assert_eq!(
            classify_state("en_orig", Some("chs_v2"), Some("en_orig"), Some("chs_v1")),
            "reverted"
        );
    }

    #[test]
    fn test_outdated_when_source_updated_but_override_intact() {
        // EN.zip still holds what we wrote (chs_v1), but CHS.zip was updated to chs_v2.
        assert_eq!(
            classify_state("chs_v1", Some("chs_v2"), Some("en_orig"), Some("chs_v1")),
            "outdated"
        );
    }

    #[test]
    fn test_modified_when_nothing_matches() {
        assert_eq!(
            classify_state("something_else", Some("chs_v2"), Some("en_orig"), Some("chs_v1")),
            "modified"
        );
    }

    #[test]
    fn test_legacy_backup_without_applied_falls_back_to_modified() {
        // Old backups have applied = None; the outdated case cannot be detected.
        assert_eq!(
            classify_state("chs_v1", Some("chs_v2"), Some("en_orig"), None),
            "modified"
        );
    }

    #[test]
    fn test_applied_takes_priority_when_source_unreadable() {
        // Source missing (None) -> not "applied"; falls through to original/applied.
        assert_eq!(
            classify_state("en_orig", None, Some("en_orig"), Some("chs_v1")),
            "reverted"
        );
        assert_eq!(
            classify_state("chs_v1", None, Some("en_orig"), Some("chs_v1")),
            "outdated"
        );
    }
}
