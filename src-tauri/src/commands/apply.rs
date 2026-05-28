use crate::core::apply_engine::{self, ApplyResult};
use crate::core::backup_manager;
use crate::core::game_detector::{GameId, GameProfile};
use crate::core::language_mapper;
use crate::core::restore_engine;

#[tauri::command]
pub fn apply_config(
    game_id: String,
    voice_lang: String,
    text_lang: String,
    resource_path: String,
    manifest_path: Option<String>,
) -> Result<ApplyResult, String> {
    let resource = std::path::Path::new(&resource_path);
    let manifest_for_profile = manifest_path.clone();
    let manifest_p = manifest_path.as_deref().map(std::path::Path::new);

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
        manifest_path: manifest_for_profile.map(std::path::PathBuf::from),
    };

    let backup_root = crate::core::backup_manager::get_backup_dir()?;
    let plan = language_mapper::generate_apply_plan(
        &game_id,
        &voice_lang,
        &text_lang,
        resource,
        &backup_root,
        manifest_p,
    )?;
    Ok(apply_engine::execute_apply(&plan, &profile))
}

/// Re-apply the most recent configuration using the latest source text pack.
///
/// Used when status detection reports `outdated`: the game updated the source
/// text pack while our override still holds the old content. We first restore
/// the latest backup (writing the *true original* voice pack back to disk and
/// reverting Steam language), then re-run a normal apply with the same
/// voice/text languages. The fresh apply backs up the true original again and
/// overwrites with the now-updated source pack, so the new backup's recorded
/// original stays correct and restore keeps working.
#[tauri::command]
pub fn reapply_config(
    game_id: String,
    resource_path: String,
    manifest_path: Option<String>,
) -> Result<ApplyResult, String> {
    let backups = backup_manager::list_backups(&game_id)?;
    let latest = backups
        .first()
        .ok_or_else(|| "No backup found to re-apply".to_string())?;

    let manifest = backup_manager::read_manifest(&latest.path)?;

    let restore = restore_engine::execute_restore(&latest.path);
    if !restore.success {
        return Ok(ApplyResult {
            success: false,
            message: format!("Failed to restore before re-apply: {}", restore.message),
            backup_path: None,
            rolled_back: false,
            steam_language_set: false,
            steam_language_warning: None,
        });
    }

    apply_config(
        game_id,
        manifest.voice_language,
        manifest.text_language,
        resource_path,
        manifest_path,
    )
}
