use crate::core::game_detector::{self, GameProfile};

#[tauri::command]
pub fn detect_games() -> Result<Vec<GameProfile>, String> {
    Ok(game_detector::detect_steam_games())
}

#[tauri::command]
pub fn validate_game_directory(path: String, game_id: String) -> Result<GameProfile, String> {
    let gid = match game_id.as_str() {
        "fh5" => game_detector::GameId::Fh5,
        "fh6" => game_detector::GameId::Fh6,
        _ => return Err(format!("Unknown game_id: {game_id}")),
    };
    game_detector::validate_game_directory(std::path::Path::new(&path), gid)
}
