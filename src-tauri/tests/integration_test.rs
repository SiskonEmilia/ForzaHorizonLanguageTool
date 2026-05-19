//! Integration tests for fh_language_combo_tool_lib.
//!
//! Groups 1-3 are **read-only** and rely on a real Steam installation with
//! both FH5 and FH6 present on this PC — marked `#[ignore]` so they don't
//! run in CI. Run them locally with `cargo test -- --ignored`.
//! Group 4 (apply + restore cycle) operates entirely inside a temp directory
//! so that no real game files are ever modified. Group 5 covers error cases
//! with synthetic / nonexistent paths.

use std::fs;
use std::path::{Path, PathBuf};

use fh_language_combo_tool_lib::core::apply_engine;
use fh_language_combo_tool_lib::core::game_detector::{self, GameId, GameProfile};
use fh_language_combo_tool_lib::core::language_mapper::{self, ApplyPlan, Operation};
use fh_language_combo_tool_lib::core::resource_scanner;
use fh_language_combo_tool_lib::core::restore_engine;

// ── Real installation paths ─────────────────────────────────────────────────

const FH5_ROOT: &str = r"E:\Program Files (x86)\Steam\steamapps\common\ForzaHorizon5";
const FH6_ROOT: &str = r"E:\Program Files (x86)\Steam\steamapps\common\ForzaHorizon6";
const FH5_RESOURCE: &str =
    r"E:\Program Files (x86)\Steam\steamapps\common\ForzaHorizon5\media\Stripped\StringTables";
const FH6_RESOURCE: &str =
    r"E:\Program Files (x86)\Steam\steamapps\common\ForzaHorizon6\media\Stripped\StringTables";

// ═══════════════════════════════════════════════════════════════════════════
// Group 1 — Steam Detection (read-only, real installation)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn test_detect_finds_fh5_and_fh6() {
    let games = game_detector::detect_steam_games();

    assert!(
        games.len() >= 2,
        "Expected at least 2 games, found {}",
        games.len()
    );

    let fh5 = games.iter().find(|g| g.game_id == GameId::Fh5);
    let fh6 = games.iter().find(|g| g.game_id == GameId::Fh6);

    assert!(fh5.is_some(), "FH5 should be detected");
    assert!(fh6.is_some(), "FH6 should be detected");

    let fh5 = fh5.unwrap();
    let fh6 = fh6.unwrap();

    assert_eq!(fh5.steam_app_id, "1551360");
    assert_eq!(fh6.steam_app_id, "2483190");

    assert!(
        fh5.resource_path
            .to_string_lossy()
            .ends_with(r"media\Stripped\StringTables"),
        "FH5 resource_path should end with media\\Stripped\\StringTables, got: {}",
        fh5.resource_path.display()
    );
    assert!(
        fh6.resource_path
            .to_string_lossy()
            .ends_with(r"media\Stripped\StringTables"),
        "FH6 resource_path should end with media\\Stripped\\StringTables, got: {}",
        fh6.resource_path.display()
    );
}

#[test]
#[ignore]
fn test_validate_fh5_real_directory() {
    let profile =
        game_detector::validate_game_directory(Path::new(FH5_ROOT), GameId::Fh5)
            .expect("validate_game_directory should succeed for real FH5 path");

    assert_eq!(profile.game_id, GameId::Fh5);
    assert_eq!(profile.display_name, "Forza Horizon 5");
    assert_eq!(profile.channel, "steam");
    assert_eq!(profile.steam_app_id, "1551360");
    assert_eq!(profile.executable_name, "ForzaHorizon5.exe");
    assert_eq!(profile.root_path, Path::new(FH5_ROOT));
    assert_eq!(profile.resource_path, Path::new(FH5_RESOURCE));
}

#[test]
#[ignore]
fn test_validate_fh6_real_directory() {
    let profile =
        game_detector::validate_game_directory(Path::new(FH6_ROOT), GameId::Fh6)
            .expect("validate_game_directory should succeed for real FH6 path");

    assert_eq!(profile.game_id, GameId::Fh6);
    assert_eq!(profile.display_name, "Forza Horizon 6");
    assert_eq!(profile.channel, "steam");
    assert_eq!(profile.steam_app_id, "2483190");
    // FH6 executable is lowercase
    assert_eq!(profile.executable_name, "forzahorizon6.exe");
    assert_eq!(profile.root_path, Path::new(FH6_ROOT));
    assert_eq!(profile.resource_path, Path::new(FH6_RESOURCE));
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 2 — Resource Scanning (read-only, real installation)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn test_scan_fh5_string_tables() {
    let packs = resource_scanner::scan_string_tables(Path::new(FH5_RESOURCE))
        .expect("scan_string_tables should succeed for real FH5 StringTables");

    assert_eq!(
        packs.len(),
        24,
        "FH5 should have exactly 24 language packs, found {}",
        packs.len()
    );

    // Verify specific language codes are present (case-insensitive lookup)
    let codes: Vec<String> = packs.iter().map(|p| p.code.to_uppercase()).collect();
    for expected in &["CHS", "EN", "JP", "BR"] {
        assert!(
            codes.contains(&expected.to_string()),
            "FH5 should contain language code {}, found: {:?}",
            expected,
            packs.iter().map(|p| &p.code).collect::<Vec<_>>()
        );
    }

    // Every pack should have a valid SHA-256 (64 hex characters) and nonzero size
    for pack in &packs {
        assert_eq!(
            pack.sha256.len(),
            64,
            "SHA-256 for {} should be 64 hex chars, got {} chars: {}",
            pack.code,
            pack.sha256.len(),
            pack.sha256
        );
        assert!(
            pack.sha256.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA-256 for {} should be all hex digits: {}",
            pack.code,
            pack.sha256
        );
        assert!(
            pack.size > 0,
            "Size for {} should be > 0, got {}",
            pack.code,
            pack.size
        );
    }

    // FH5 peculiarity: "br" and "cz" are lowercase in the actual filenames
    let br_pack = packs.iter().find(|p| p.code.eq_ignore_ascii_case("br"));
    assert!(br_pack.is_some(), "FH5 should have a BR pack");
    assert_eq!(
        br_pack.unwrap().code, "br",
        "FH5 BR code should be lowercase 'br' (from br.zip)"
    );

    let cz_pack = packs.iter().find(|p| p.code.eq_ignore_ascii_case("cz"));
    assert!(cz_pack.is_some(), "FH5 should have a CZ pack");
    assert_eq!(
        cz_pack.unwrap().code, "cz",
        "FH5 CZ code should be lowercase 'cz' (from cz.zip)"
    );
}

#[test]
#[ignore]
fn test_scan_fh6_string_tables() {
    let packs = resource_scanner::scan_string_tables(Path::new(FH6_RESOURCE))
        .expect("scan_string_tables should succeed for real FH6 StringTables");

    assert_eq!(
        packs.len(),
        24,
        "FH6 should have exactly 24 language packs, found {}",
        packs.len()
    );

    // FH6 should have uppercase "BR" (from BR.zip), unlike FH5's lowercase "br"
    let br_pack = packs.iter().find(|p| p.code.eq_ignore_ascii_case("br"));
    assert!(br_pack.is_some(), "FH6 should have a BR pack");
    assert_eq!(
        br_pack.unwrap().code, "BR",
        "FH6 BR code should be uppercase 'BR' (from BR.zip)"
    );

    // FH6 CZ should also be uppercase
    let cz_pack = packs.iter().find(|p| p.code.eq_ignore_ascii_case("cz"));
    assert!(cz_pack.is_some(), "FH6 should have a CZ pack");
    assert_eq!(
        cz_pack.unwrap().code, "CZ",
        "FH6 CZ code should be uppercase 'CZ' (from CZ.zip)"
    );
}

#[test]
#[ignore]
fn test_fh5_filename_case_preservation() {
    let fh5_res = Path::new(FH5_RESOURCE);

    // "br" input -> should find "br.zip" (FH5 has lowercase br.zip)
    let result = language_mapper::resolve_filename("br", fh5_res).unwrap();
    assert_eq!(result, "br.zip", "resolve_filename('br') on FH5 should return 'br.zip'");

    // "BR" input -> should still find "br.zip" (case-insensitive match)
    let result = language_mapper::resolve_filename("BR", fh5_res).unwrap();
    assert_eq!(result, "br.zip", "resolve_filename('BR') on FH5 should return 'br.zip'");

    // "CHS" input -> should find "CHS.zip" (uppercase in FH5)
    let result = language_mapper::resolve_filename("CHS", fh5_res).unwrap();
    assert_eq!(result, "CHS.zip", "resolve_filename('CHS') on FH5 should return 'CHS.zip'");

    // "cz" input -> should find "cz.zip" (FH5 has lowercase cz.zip)
    let result = language_mapper::resolve_filename("cz", fh5_res).unwrap();
    assert_eq!(result, "cz.zip", "resolve_filename('cz') on FH5 should return 'cz.zip'");
}

#[test]
#[ignore]
fn test_fh6_filename_case_all_uppercase() {
    let fh6_res = Path::new(FH6_RESOURCE);

    // "br" input -> should find "BR.zip" (FH6 has all uppercase)
    let result = language_mapper::resolve_filename("br", fh6_res).unwrap();
    assert_eq!(result, "BR.zip", "resolve_filename('br') on FH6 should return 'BR.zip'");

    // "chs" input -> should find "CHS.zip"
    let result = language_mapper::resolve_filename("chs", fh6_res).unwrap();
    assert_eq!(result, "CHS.zip", "resolve_filename('chs') on FH6 should return 'CHS.zip'");

    // "CZ" input -> should find "CZ.zip" (uppercase in FH6)
    let result = language_mapper::resolve_filename("CZ", fh6_res).unwrap();
    assert_eq!(result, "CZ.zip", "resolve_filename('CZ') on FH6 should return 'CZ.zip'");
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 3 — Apply Plan Generation (read-only)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn test_generate_plan_jp_voice_chs_text_fh5() {
    let fh5_res = Path::new(FH5_RESOURCE);
    let backup_root = std::env::temp_dir().join("fh_integ_plan_test");
    let _ = fs::create_dir_all(&backup_root);

    let plan = language_mapper::generate_apply_plan("fh5", "JP", "CHS", fh5_res, &backup_root)
        .expect("generate_apply_plan should succeed for JP voice + CHS text on FH5");

    assert_eq!(plan.game_id, "fh5");
    assert_eq!(plan.voice_language, "JP");
    assert_eq!(plan.text_language, "CHS");
    // FH5 CHS.zip is uppercase, JP.zip is uppercase
    assert_eq!(plan.source_file, "CHS.zip", "Source (text) file should be CHS.zip");
    assert_eq!(plan.target_file, "JP.zip", "Target (voice) file should be JP.zip");

    assert_eq!(
        plan.operations.len(),
        2,
        "Plan should have exactly 2 operations (backup + copy_replace)"
    );

    let backup_op = &plan.operations[0];
    assert_eq!(backup_op.op_type, "backup");
    assert!(
        backup_op.from.contains("JP.zip"),
        "Backup 'from' should contain JP.zip, got: {}",
        backup_op.from
    );

    let copy_op = &plan.operations[1];
    assert_eq!(copy_op.op_type, "copy_replace");
    assert!(
        copy_op.from.contains("CHS.zip"),
        "Copy 'from' should contain CHS.zip, got: {}",
        copy_op.from
    );
    assert!(
        copy_op.to.contains("JP.zip"),
        "Copy 'to' should contain JP.zip, got: {}",
        copy_op.to
    );

    let _ = fs::remove_dir_all(&backup_root);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 4 — Full Apply + Restore Cycle (temp directory, no real game files)
// ═══════════════════════════════════════════════════════════════════════════

/// Helper: build a unique temp directory for an integration test.
fn make_integ_temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("fh_integ_test")
        .join(format!("{}_{}", label, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("Failed to create integration test temp dir");
    dir
}

#[test]
fn test_full_apply_and_restore_cycle() {
    // 1. Set up a temp directory that mimics a game installation with StringTables
    let tmp = make_integ_temp_dir("apply_restore_cycle");
    let resource_dir = tmp.join("media").join("Stripped").join("StringTables");
    fs::create_dir_all(&resource_dir).unwrap();

    let en_content = b"original-english-voice-data-12345";
    let chs_content = b"chinese-text-data-67890";

    let en_file = resource_dir.join("EN.zip");
    let chs_file = resource_dir.join("CHS.zip");
    fs::write(&en_file, en_content).unwrap();
    fs::write(&chs_file, chs_content).unwrap();

    // Verify the two files have different content before applying
    assert_ne!(
        fs::read(&en_file).unwrap(),
        fs::read(&chs_file).unwrap(),
        "EN.zip and CHS.zip should have different content before apply"
    );

    // 2. Build an ApplyPlan for EN voice + CHS text.
    //    This means: backup EN.zip, then overwrite EN.zip with CHS.zip contents.
    let plan = ApplyPlan {
        game_id: "fh5".into(),
        voice_language: "EN".into(),
        text_language: "CHS".into(),
        source_file: "CHS.zip".into(),
        target_file: "EN.zip".into(),
        operations: vec![
            Operation {
                op_type: "backup".into(),
                from: en_file.to_string_lossy().into_owned(),
                to: "will-be-determined-by-backup-manager".into(),
                description: "Backup EN.zip".into(),
            },
            Operation {
                op_type: "copy_replace".into(),
                from: chs_file.to_string_lossy().into_owned(),
                to: en_file.to_string_lossy().into_owned(),
                description: "Copy CHS.zip -> EN.zip".into(),
            },
        ],
    };

    // 3. Build a GameProfile pointing to our temp directory
    let profile = GameProfile {
        game_id: GameId::Fh5,
        display_name: "Forza Horizon 5".into(),
        channel: "steam".into(),
        steam_app_id: "1551360".into(),
        root_path: tmp.clone(),
        resource_path: resource_dir.clone(),
        executable_name: "ForzaHorizon5.exe".into(),
    };

    // 4. Execute apply (use temp backup root to avoid polluting real backup dir)
    let backup_root = make_integ_temp_dir("apply_restore_backup");
    let apply_result = apply_engine::execute_apply_to(&plan, &profile, &backup_root);

    assert!(
        apply_result.success,
        "Apply should succeed, got: {}",
        apply_result.message
    );
    assert!(
        apply_result.backup_path.is_some(),
        "Apply should produce a backup_path"
    );
    assert!(!apply_result.rolled_back, "Apply should not have rolled back");

    let backup_path_str = apply_result.backup_path.as_ref().unwrap();
    let backup_path = PathBuf::from(backup_path_str);

    // 5. Verify EN.zip now has the same content as CHS.zip (the text language)
    let en_after_apply = fs::read(&en_file).unwrap();
    assert_eq!(
        en_after_apply, chs_content,
        "After apply, EN.zip should contain CHS.zip content"
    );

    // CHS.zip should be unchanged
    let chs_after_apply = fs::read(&chs_file).unwrap();
    assert_eq!(
        chs_after_apply, chs_content,
        "CHS.zip should remain unchanged after apply"
    );

    // 6. Execute restore using the backup path
    let restore_result = restore_engine::execute_restore(&backup_path);

    assert!(
        restore_result.success,
        "Restore should succeed, got: {}",
        restore_result.message
    );

    // 7. Verify EN.zip is back to its original content
    let en_after_restore = fs::read(&en_file).unwrap();
    assert_eq!(
        en_after_restore, en_content,
        "After restore, EN.zip should be back to original content"
    );

    // 8. Clean up
    let _ = fs::remove_dir_all(&tmp);
    let _ = fs::remove_dir_all(&backup_root);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 5 — Error Cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_validate_rejects_nonexistent_directory() {
    let fake_path = Path::new(r"Z:\nonexistent\path\ForzaHorizon5");
    let result = game_detector::validate_game_directory(fake_path, GameId::Fh5);
    assert!(
        result.is_err(),
        "validate_game_directory should reject a nonexistent directory"
    );
}

#[test]
fn test_scan_nonexistent_directory_errors() {
    let fake_path = Path::new(r"Z:\nonexistent\path\StringTables");
    let result = resource_scanner::scan_string_tables(fake_path);
    assert!(
        result.is_err(),
        "scan_string_tables should return Err for a nonexistent directory"
    );
}

#[test]
fn test_plan_same_language_rejected() {
    let fh5_res = Path::new(FH5_RESOURCE);
    let backup_root = std::env::temp_dir().join("fh_integ_same_lang_test");
    let _ = fs::create_dir_all(&backup_root);

    let result = language_mapper::generate_apply_plan("fh5", "EN", "en", fh5_res, &backup_root);
    assert!(
        result.is_err(),
        "generate_apply_plan should reject same voice and text language"
    );

    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("must be different"),
        "Error should mention languages must be different, got: {}",
        err_msg
    );

    let _ = fs::remove_dir_all(&backup_root);
}
