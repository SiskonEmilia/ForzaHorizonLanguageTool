use crate::core::apply_engine::{self, ApplyResult};
use crate::core::game_detector::{GameId, GameProfile};
use crate::core::language_mapper;

#[tauri::command]
pub fn apply_config(
    game_id: String,
    voice_lang: String,
    text_lang: String,
    resource_path: String,
) -> Result<ApplyResult, String> {
    let resource = std::path::Path::new(&resource_path);

    let gid = match game_id.as_str() {
        "fh5" => GameId::Fh5,
        "fh6" => GameId::Fh6,
        _ => return Err(format!("Unknown game_id: {game_id}")),
    };

    let root_path = resource
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| {
            format!(
                "Cannot derive game root from resource path: {}",
                resource_path
            )
        })?;

    let profile = GameProfile {
        game_id: gid,
        display_name: match gid {
            GameId::Fh5 => "Forza Horizon 5".into(),
            GameId::Fh6 => "Forza Horizon 6".into(),
        },
        channel: "steam".into(),
        steam_app_id: match gid {
            GameId::Fh5 => "1551360".into(),
            GameId::Fh6 => "2483190".into(),
        },
        root_path: root_path.to_path_buf(),
        resource_path: resource.to_path_buf(),
        executable_name: match gid {
            GameId::Fh5 => "ForzaHorizon5.exe".into(),
            GameId::Fh6 => "forzahorizon6.exe".into(),
        },
    };

    let backup_root = crate::core::backup_manager::get_backup_dir()?;
    let plan = language_mapper::generate_apply_plan(
        &game_id,
        &voice_lang,
        &text_lang,
        resource,
        &backup_root,
    )?;
    Ok(apply_engine::execute_apply(&plan, &profile))
}
