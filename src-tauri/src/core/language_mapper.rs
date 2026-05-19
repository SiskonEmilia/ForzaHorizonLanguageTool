use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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

// TODO: implement
pub fn generate_apply_plan(
    _game_id: &str,
    _voice_lang: &str,
    _text_lang: &str,
    _resource_path: &std::path::Path,
    _backup_root: &std::path::Path,
) -> Result<ApplyPlan, String> {
    Err("not implemented".into())
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
