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

    let state = if source_hash.as_deref() == Some(current_hash.as_str()) {
        "applied"
    } else if original_hash == Some(current_hash.as_str()) {
        "reverted"
    } else {
        "modified"
    };

    Ok(ConfigStatus {
        state: state.into(),
        game_id: Some(manifest.game),
        voice_language: Some(manifest.voice_language),
        text_language: Some(manifest.text_language),
        last_applied: Some(manifest.created_at),
    })
}
