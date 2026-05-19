use crate::core::resource_scanner::{self, LanguagePack};

#[tauri::command]
pub fn scan_language_packs(resource_path: String) -> Result<Vec<LanguagePack>, String> {
    resource_scanner::scan_string_tables(std::path::Path::new(&resource_path))
}
