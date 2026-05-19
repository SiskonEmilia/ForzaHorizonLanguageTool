use crate::core::backup_manager::{self, BackupInfo};
use crate::core::restore_engine::{self, RestoreResult};

#[tauri::command]
pub fn list_backups(game_id: String) -> Result<Vec<BackupInfo>, String> {
    backup_manager::list_backups(&game_id)
}

#[tauri::command]
pub fn restore_backup(backup_path: String) -> Result<RestoreResult, String> {
    Ok(restore_engine::execute_restore(std::path::Path::new(&backup_path)))
}
