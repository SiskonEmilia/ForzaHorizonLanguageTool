use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    #[serde(default)]
    pub manifest_path: Option<String>,
    #[serde(default)]
    pub original_steam_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFileEntry {
    pub path: String,
    pub original_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub id: String,
    pub game: String,
    pub voice_language: String,
    pub text_language: String,
    pub created_at: String,
    pub path: PathBuf,
    pub valid: bool,
}

use super::resource_scanner;

pub fn get_backup_dir() -> Result<PathBuf, String> {
    let local_app_data =
        dirs::data_local_dir().ok_or_else(|| "LOCALAPPDATA directory not available".to_string())?;
    let backup_dir = local_app_data
        .join("FHLanguageComboTool")
        .join("backups");
    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    Ok(backup_dir)
}

pub fn create_backup_to(
    backup_root: &Path,
    game_id: &str,
    channel: &str,
    game_root: &Path,
    resource_dir: &Path,
    voice_lang: &str,
    text_lang: &str,
    target_file: &Path,
    source_file: &str,
    acf_manifest_path: Option<&Path>,
    original_steam_language: Option<&str>,
) -> Result<PathBuf, String> {
    let now = chrono::Local::now();
    let dir_name = format!(
        "{}_{}_voice_{}_text",
        now.format("%Y%m%d_%H%M%S"),
        voice_lang,
        text_lang
    );
    let backup_dir = backup_root.join(game_id).join(&dir_name);
    let original_dir = backup_dir.join("original");

    fs::create_dir_all(&original_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let cleanup = |msg: String| -> String {
        let _ = fs::remove_dir_all(&backup_dir);
        msg
    };

    let file_name = target_file
        .file_name()
        .ok_or_else(|| cleanup("Target file has no filename".to_string()))?;
    let dest = original_dir.join(file_name);

    let source_hash = resource_scanner::compute_sha256(target_file).map_err(|e| cleanup(e))?;

    fs::copy(target_file, &dest).map_err(|e| cleanup(format!("Failed to copy file: {}", e)))?;

    let dest_hash = resource_scanner::compute_sha256(&dest).map_err(|e| cleanup(e))?;
    if source_hash != dest_hash {
        return Err(cleanup(format!(
            "SHA-256 mismatch after copy: source={}, dest={}",
            source_hash, dest_hash
        )));
    }

    let manifest = BackupManifest {
        tool_version: "1.0.0".to_string(),
        game: game_id.to_string(),
        channel: channel.to_string(),
        game_root: game_root.to_string_lossy().to_string(),
        resource_directory: resource_dir.to_string_lossy().to_string(),
        voice_language: voice_lang.to_string(),
        text_language: text_lang.to_string(),
        target_file: file_name.to_string_lossy().to_string(),
        source_file: source_file.to_string(),
        created_at: now.to_rfc3339(),
        files: vec![BackupFileEntry {
            path: file_name.to_string_lossy().to_string(),
            original_sha256: source_hash,
        }],
        manifest_path: acf_manifest_path.map(|p| p.to_string_lossy().to_string()),
        original_steam_language: original_steam_language.map(|s| s.to_string()),
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| cleanup(format!("Failed to serialize manifest: {}", e)))?;
    fs::write(backup_dir.join("manifest.json"), manifest_json)
        .map_err(|e| cleanup(format!("Failed to write manifest: {}", e)))?;

    Ok(backup_dir)
}

pub fn list_backups(game_id: &str) -> Result<Vec<BackupInfo>, String> {
    let backup_root = get_backup_dir()?;
    list_backups_from(&backup_root, game_id)
}

fn list_backups_from(backup_root: &Path, game_id: &str) -> Result<Vec<BackupInfo>, String> {
    let game_dir = backup_root.join(game_id);
    if !game_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&game_dir)
        .map_err(|e| format!("Failed to read backup directory: {}", e))?;

    let mut backups: Vec<BackupInfo> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("manifest.json");
        let manifest = match read_manifest(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let has_original_files = manifest.files.iter().all(|f| {
            path.join("original").join(&f.path).exists()
        });

        let id = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        backups.push(BackupInfo {
            id,
            game: manifest.game,
            voice_language: manifest.voice_language,
            text_language: manifest.text_language,
            created_at: manifest.created_at,
            path: path.clone(),
            valid: manifest_path.exists() && has_original_files,
        });
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(backups)
}

pub fn read_manifest(backup_path: &Path) -> Result<BackupManifest, String> {
    let manifest_path = backup_path.join("manifest.json");
    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse manifest: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn make_test_dir(label: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join("fhlct_test")
            .join(format!("{}_{}", label, id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("Failed to create test dir");
        dir
    }

    #[test]
    fn test_get_backup_dir_contains_app_name() {
        let dir = get_backup_dir().expect("get_backup_dir should succeed");
        let path_str = dir.to_string_lossy();
        assert!(
            path_str.contains("FHLanguageComboTool"),
            "Backup dir should contain FHLanguageComboTool: {}",
            path_str
        );
        assert!(dir.exists(), "Backup dir should be created");
    }

    #[test]
    fn test_create_backup_writes_manifest_and_copies_file() {
        let tmp = make_test_dir("create_backup");
        let backup_root = tmp.join("backups");
        let resource_dir = tmp.join("resources");
        fs::create_dir_all(&resource_dir).unwrap();

        let target_file = resource_dir.join("StringTable_en-US.txt");
        fs::write(&target_file, "test content for backup").unwrap();

        let game_root = tmp.join("game");
        fs::create_dir_all(&game_root).unwrap();

        let result = create_backup_to(
            &backup_root,
            "fh5",
            "steam",
            &game_root,
            &resource_dir,
            "en-US",
            "ja-JP",
            &target_file,
            "StringTable_ja-JP.txt",
            None,
            None,
        );

        let backup_dir = result.expect("create_backup_to should succeed");
        assert!(backup_dir.exists());

        let manifest_path = backup_dir.join("manifest.json");
        assert!(manifest_path.exists(), "manifest.json should exist");

        let manifest = read_manifest(&backup_dir).expect("manifest should be readable");
        assert_eq!(manifest.tool_version, "1.0.0");
        assert_eq!(manifest.game, "fh5");
        assert_eq!(manifest.channel, "steam");
        assert_eq!(manifest.voice_language, "en-US");
        assert_eq!(manifest.text_language, "ja-JP");
        assert_eq!(manifest.source_file, "StringTable_ja-JP.txt");
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.files[0].path, "StringTable_en-US.txt");

        let copied = backup_dir.join("original").join("StringTable_en-US.txt");
        assert!(copied.exists(), "Original file should be copied");
        let content = fs::read_to_string(&copied).unwrap();
        assert_eq!(content, "test content for backup");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_list_backups_empty_dir() {
        let tmp = make_test_dir("list_empty");
        let backup_root = tmp.join("backups");
        fs::create_dir_all(&backup_root).unwrap();

        let result = list_backups_from(&backup_root, "fh5").expect("list_backups should succeed");
        assert!(result.is_empty(), "Should return empty vec for missing game dir");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_read_manifest_valid_json() {
        let tmp = make_test_dir("manifest_valid");
        let manifest = BackupManifest {
            tool_version: "1.0.0".to_string(),
            game: "fh5".to_string(),
            channel: "steam".to_string(),
            game_root: "C:\\Games\\FH5".to_string(),
            resource_directory: "C:\\Games\\FH5\\media\\stringtables".to_string(),
            voice_language: "en-US".to_string(),
            text_language: "ja-JP".to_string(),
            target_file: "StringTable_en-US.txt".to_string(),
            source_file: "StringTable_ja-JP.txt".to_string(),
            created_at: "2025-01-01T00:00:00+09:00".to_string(),
            files: vec![BackupFileEntry {
                path: "StringTable_en-US.txt".to_string(),
                original_sha256: "abc123".to_string(),
            }],
            manifest_path: None,
            original_steam_language: None,
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(tmp.join("manifest.json"), json).unwrap();

        let result = read_manifest(&tmp).expect("Should parse valid manifest");
        assert_eq!(result.game, "fh5");
        assert_eq!(result.voice_language, "en-US");
        assert_eq!(result.text_language, "ja-JP");
        assert_eq!(result.files.len(), 1);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_read_manifest_invalid_json() {
        let tmp = make_test_dir("manifest_invalid");
        fs::write(tmp.join("manifest.json"), "not valid json{{{").unwrap();

        let result = read_manifest(&tmp);
        assert!(result.is_err(), "Should return error for invalid JSON");

        let _ = fs::remove_dir_all(&tmp);
    }
}
