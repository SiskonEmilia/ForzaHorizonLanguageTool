use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ConfigStatus {
    pub state: String,
    pub game_id: Option<String>,
    pub voice_language: Option<String>,
    pub text_language: Option<String>,
    pub last_applied: Option<String>,
}

#[tauri::command]
pub fn get_status(game_id: String, resource_path: String) -> Result<ConfigStatus, String> {
    let _ = (&game_id, &resource_path);
    Ok(ConfigStatus {
        state: "none".into(),
        game_id: None,
        voice_language: None,
        text_language: None,
        last_applied: None,
    })
}

#[tauri::command]
pub fn check_game_running(game_id: String) -> Result<bool, String> {
    let process_name = match game_id.as_str() {
        "fh5" => "ForzaHorizon5.exe",
        "fh6" => "forzahorizon6.exe",
        _ => return Err(format!("Unknown game_id: {game_id}")),
    };
    let _ = process_name;
    Ok(false) // TODO: implement
}
