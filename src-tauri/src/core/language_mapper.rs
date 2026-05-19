use std::path::Path;

use chrono::Local;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPlan {
    pub game_id: String,
    pub voice_language: String,
    pub text_language: String,
    pub source_file: String,
    pub target_file: String,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Operation {
    #[serde(rename = "type")]
    pub op_type: String,
    pub from: String,
    pub to: String,
    pub description: String,
}

pub fn get_display_name(code: &str) -> &'static str {
    match code.to_uppercase().as_str() {
        "EN" => "English",
        "GB" => "English (UK)",
        "JP" => "日本語",
        "CHS" => "简体中文",
        "CHT" => "繁體中文",
        "FR" => "Français",
        "DE" => "Deutsch",
        "ES" => "Español",
        "MX" => "Español (MX)",
        "IT" => "Italiano",
        "PT" => "Português",
        "BR" => "Português (BR)",
        "KO" => "한국어",
        "RU" => "Русский",
        "PL" => "Polski",
        "NL" => "Nederlands",
        "TR" => "Türkçe",
        "DK" => "Dansk",
        "SV" => "Svenska",
        "NO" => "Norsk",
        "FI" => "Suomi",
        "CZ" => "Čeština",
        "HU" => "Magyar",
        "EL" => "Ελληνικά",
        _ => "Unknown",
    }
}

pub fn resolve_filename(code: &str, resource_path: &Path) -> Result<String, String> {
    let entries = std::fs::read_dir(resource_path)
        .map_err(|e| format!("Failed to read directory {}: {}", resource_path.display(), e))?;

    let code_upper = code.to_uppercase();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("zip") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.to_uppercase() == code_upper {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .ok_or_else(|| "Invalid filename".to_string())?;
                    return Ok(filename.to_string());
                }
            }
        }
    }

    Err(format!(
        "No .zip file found for language code '{}' in {}",
        code,
        resource_path.display()
    ))
}

pub fn generate_apply_plan(
    game_id: &str,
    voice_lang: &str,
    text_lang: &str,
    resource_path: &Path,
    backup_root: &Path,
) -> Result<ApplyPlan, String> {
    if voice_lang.eq_ignore_ascii_case(text_lang) {
        return Err("Voice language and text language must be different".into());
    }

    let source_file = resolve_filename(text_lang, resource_path)?;
    let target_file = resolve_filename(voice_lang, resource_path)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let voice_upper = voice_lang.to_uppercase();
    let text_upper = text_lang.to_uppercase();
    let backup_dir = backup_root
        .join(game_id)
        .join(format!(
            "{}_{}_voice_{}_text",
            timestamp, voice_upper, text_upper
        ))
        .join("original");

    let resource_source = resource_path.join(&source_file);
    let resource_target = resource_path.join(&target_file);
    let backup_target = backup_dir.join(&target_file);

    let operations = vec![
        Operation {
            op_type: "backup".into(),
            from: resource_target.to_string_lossy().into_owned(),
            to: backup_target.to_string_lossy().into_owned(),
            description: format!("Backup {}", target_file),
        },
        Operation {
            op_type: "copy_replace".into(),
            from: resource_source.to_string_lossy().into_owned(),
            to: resource_target.to_string_lossy().into_owned(),
            description: format!("Copy {} → {}", source_file, target_file),
        },
    ];

    Ok(ApplyPlan {
        game_id: game_id.to_string(),
        voice_language: voice_upper,
        text_language: text_upper,
        source_file,
        target_file,
        operations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_temp_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("fh_lang_mapper_test")
            .join(format!("{}_{}", label, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_display_name_known_codes() {
        assert_eq!(get_display_name("EN"), "English");
        assert_eq!(get_display_name("CHS"), "简体中文");
        assert_eq!(get_display_name("JP"), "日本語");
        assert_eq!(get_display_name("BR"), "Português (BR)");
    }

    #[test]
    fn test_display_name_unknown_code() {
        assert_eq!(get_display_name("ZZ"), "Unknown");
    }

    #[test]
    fn test_display_name_case_insensitive() {
        assert_eq!(get_display_name("en"), "English");
        assert_eq!(get_display_name("chs"), "简体中文");
        assert_eq!(get_display_name("En"), "English");
    }

    #[test]
    fn test_resolve_filename_preserves_case() {
        let dir = make_temp_dir("resolve");
        fs::write(dir.join("CHS.zip"), b"").unwrap();
        fs::write(dir.join("br.zip"), b"").unwrap();

        let result = resolve_filename("chs", &dir).unwrap();
        assert_eq!(result, "CHS.zip");

        let result = resolve_filename("BR", &dir).unwrap();
        assert_eq!(result, "br.zip");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_generate_apply_plan_same_language_error() {
        let dir = make_temp_dir("same_lang");
        let backup = make_temp_dir("same_lang_backup");

        let result = generate_apply_plan("fh5", "EN", "en", &dir, &backup);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Voice language and text language must be different"));

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&backup);
    }

    #[test]
    fn test_generate_apply_plan_operations() {
        let dir = make_temp_dir("plan_ops");
        let backup = make_temp_dir("plan_ops_backup");
        fs::write(dir.join("EN.zip"), b"").unwrap();
        fs::write(dir.join("CHS.zip"), b"").unwrap();

        let plan = generate_apply_plan("fh5", "EN", "CHS", &dir, &backup).unwrap();

        assert_eq!(plan.game_id, "fh5");
        assert_eq!(plan.voice_language, "EN");
        assert_eq!(plan.text_language, "CHS");
        assert_eq!(plan.source_file, "CHS.zip");
        assert_eq!(plan.target_file, "EN.zip");
        assert_eq!(plan.operations.len(), 2);

        assert_eq!(plan.operations[0].op_type, "backup");
        assert!(plan.operations[0].from.contains("EN.zip"));
        assert!(plan.operations[0].to.contains("original"));
        assert!(plan.operations[0].to.contains("EN.zip"));
        assert_eq!(plan.operations[0].description, "Backup EN.zip");

        assert_eq!(plan.operations[1].op_type, "copy_replace");
        assert!(plan.operations[1].from.contains("CHS.zip"));
        assert!(plan.operations[1].to.contains("EN.zip"));
        assert_eq!(plan.operations[1].description, "Copy CHS.zip → EN.zip");

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&backup);
    }
}
