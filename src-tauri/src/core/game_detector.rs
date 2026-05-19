use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct GameProfile {
    pub game_id: GameId,
    pub display_name: String,
    pub channel: String,
    pub steam_app_id: String,
    pub root_path: PathBuf,
    pub resource_path: PathBuf,
    pub executable_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GameId {
    Fh5,
    Fh6,
}

// TODO: implement
pub fn detect_steam_games() -> Vec<GameProfile> {
    vec![]
}

// TODO: implement
pub fn validate_game_directory(_path: &std::path::Path, _game_id: GameId) -> Result<GameProfile, String> {
    Err("not implemented".into())
}
