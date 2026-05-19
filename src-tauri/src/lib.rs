pub mod commands;
pub mod core;

use commands::{apply, detect, restore, scan, status};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect::detect_games,
            detect::validate_game_directory,
            scan::scan_language_packs,
            apply::apply_config,
            restore::restore_backup,
            restore::list_backups,
            status::get_status,
            status::check_game_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
