use crate::core::apply_engine::{self, ApplyResult};
use crate::core::language_mapper;

#[tauri::command]
pub fn apply_config(
    game_id: String,
    voice_lang: String,
    text_lang: String,
    resource_path: String,
) -> Result<ApplyResult, String> {
    let backup_root = crate::core::backup_manager::get_backup_dir()?;
    let plan = language_mapper::generate_apply_plan(
        &game_id,
        &voice_lang,
        &text_lang,
        std::path::Path::new(&resource_path),
        &backup_root,
    )?;
    Ok(apply_engine::execute_apply(&plan))
}
