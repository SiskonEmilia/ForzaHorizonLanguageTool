# Module Interfaces — Implementation Contract

This document defines the exact interface each Rust module must implement.
SubAgents must follow these signatures exactly. All modules live under `src-tauri/src/core/`.

---

## Shared Types

All types use `serde::Serialize` (and `Deserialize` where noted). JSON field names use camelCase via `#[serde(rename_all = "camelCase")]`.

---

## Module 1: game_detector

**File**: `core/game_detector.rs`
**Dependencies**: `winreg`, `std::fs`, `std::path`
**No dependency on other core modules.**

### Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameId { Fh5, Fh6 }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameProfile {
    pub game_id: GameId,
    pub display_name: String,
    pub channel: String,
    pub steam_app_id: String,
    pub root_path: PathBuf,
    pub resource_path: PathBuf,
    pub executable_name: String,
}
```

### Constants

```rust
pub const FH5_STEAM_APP_ID: &str = "1551360";
pub const FH6_STEAM_APP_ID: &str = "2483190";
pub const FH5_INSTALL_DIR: &str = "ForzaHorizon5";
pub const FH6_INSTALL_DIR: &str = "ForzaHorizon6";
pub const FH5_EXECUTABLE: &str = "ForzaHorizon5.exe";
pub const FH6_EXECUTABLE: &str = "forzahorizon6.exe";  // lowercase!
pub const RESOURCE_SUBPATH: &str = r"media\Stripped\StringTables";
```

### Functions

```rust
/// Detect all installed FH games from Steam libraries.
/// 1. Read HKCU\SOFTWARE\Valve\Steam → SteamPath
/// 2. Parse {SteamPath}/steamapps/libraryfolders.vdf
/// 3. For each library, check appmanifest_1551360.acf and appmanifest_2483190.acf
/// 4. Validate each detected directory
/// Returns empty vec if nothing found (never errors).
pub fn detect_steam_games() -> Vec<GameProfile>

/// Validate a user-provided directory as a valid game install.
/// Checks: executable exists, RESOURCE_SUBPATH exists, ≥2 .zip files in StringTables.
/// Rejects system dirs, disk roots, user home.
pub fn validate_game_directory(path: &Path, game_id: GameId) -> Result<GameProfile, String>
```

### VDF/ACF Parsing

libraryfolders.vdf and appmanifest_*.acf are Valve's simple key-value format:
```
"key"    "value"
"key"
{
    "nested_key"    "nested_value"
}
```
Implement a minimal parser — do NOT pull in external VDF crate. Only need to extract:
- From libraryfolders.vdf: each library's `path`
- From appmanifest_*.acf: `installdir`, `UserConfig.language`

### Test Requirements

- Unit test: parse a sample libraryfolders.vdf string
- Unit test: parse a sample appmanifest .acf string
- Unit test: validate_game_directory rejects empty dir
- Unit test: validate_game_directory rejects disk root (e.g. `C:\`)

---

## Module 2: resource_scanner

**File**: `core/resource_scanner.rs`
**Dependencies**: `std::fs`, `sha2`, `chrono`
**Depends on**: `language_mapper::get_display_name`

### Types

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguagePack {
    pub code: String,
    pub display_name: String,
    pub file_name: String,
    pub path: PathBuf,
    pub size: u64,
    pub sha256: String,
    pub modified_at: String,   // ISO 8601
    pub readable: bool,
    pub writable: bool,
}
```

### Functions

```rust
/// Scan the StringTables directory for .zip files.
/// For each .zip: extract language code from filename (strip .zip, case-insensitive),
/// compute SHA-256, get file size and modification time, check read/write permissions.
/// Returns sorted by language code.
pub fn scan_string_tables(resource_path: &Path) -> Result<Vec<LanguagePack>, String>

/// Compute SHA-256 of a single file. Reusable by other modules.
pub fn compute_sha256(path: &Path) -> Result<String, String>
```

### Important

- Language code extraction: `CHS.zip` → `"CHS"`, `br.zip` → `"br"` (preserve original case!)
- Use `language_mapper::get_display_name` for display name lookup (case-insensitive)
- Modified time as ISO 8601 string

### Test Requirements

- Unit test: compute_sha256 on a known byte sequence (use tempfile)
- Unit test: scan_string_tables with a temp dir containing 3 fake .zip files
- Unit test: scan_string_tables on empty dir returns empty vec (not error)
- Unit test: code extraction preserves case

---

## Module 3: language_mapper

**File**: `core/language_mapper.rs`
**Dependencies**: `std::path`
**No dependency on other core modules.**

### Types

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPlan {
    pub game_id: String,
    pub voice_language: String,
    pub text_language: String,
    pub source_file: String,
    pub target_file: String,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Operation {
    #[serde(rename = "type")]
    pub op_type: String,       // "backup" or "copy_replace"
    pub from: String,
    pub to: String,
    pub description: String,
}
```

### Functions

```rust
/// Map language code to display name. Case-insensitive lookup.
pub fn get_display_name(code: &str) -> &'static str

/// Map voice language code to the StringTables filename it maps to.
/// Must resolve the actual filename from the resource_path directory
/// (case-insensitive match), since FH5 has `br.zip` but FH6 has `BR.zip`.
pub fn resolve_filename(code: &str, resource_path: &Path) -> Result<String, String>

/// Generate the apply plan.
/// - source_file = resolve_filename(text_lang, resource_path)
/// - target_file = resolve_filename(voice_lang, resource_path)
/// - operations: [{backup target_file}, {copy_replace source→target}]
/// Error if voice_lang == text_lang, or if either file not found.
pub fn generate_apply_plan(
    game_id: &str,
    voice_lang: &str,
    text_lang: &str,
    resource_path: &Path,
    backup_root: &Path,
) -> Result<ApplyPlan, String>
```

### Test Requirements

- Unit test: get_display_name for known codes (EN, CHS, JP)
- Unit test: get_display_name for unknown code returns "Unknown"
- Unit test: generate_apply_plan errors when voice == text
- Unit test: resolve_filename case-insensitive matching

---

## Module 4: backup_manager

**File**: `core/backup_manager.rs`
**Dependencies**: `std::fs`, `chrono`, `serde_json`, `dirs`
**Depends on**: `resource_scanner::compute_sha256`

### Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub tool_version: String,
    pub game: String,
    pub channel: String,
    pub game_root: String,
    pub resource_directory: String,
    pub voice_language: String,
    pub text_language: String,
    pub target_file: String,
    pub source_file: String,
    pub created_at: String,         // ISO 8601
    pub files: Vec<BackupFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFileEntry {
    pub path: String,
    pub original_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub id: String,
    pub game: String,
    pub voice_language: String,
    pub text_language: String,
    pub created_at: String,
    pub path: PathBuf,
    pub valid: bool,
}
```

### Functions

```rust
/// Returns %LOCALAPPDATA%/FHLanguageComboTool/backups/
pub fn get_backup_dir() -> Result<PathBuf, String>

/// Create a backup of target_file before it gets overwritten.
/// 1. Create dir: {backup_dir}/{game_id}/{timestamp}_{voice}_voice_{text}_text/original/
/// 2. Copy target_file into original/
/// 3. Compute SHA-256 of the copied file and verify it matches source
/// 4. Write manifest.json
/// Returns the backup directory path.
pub fn create_backup(
    game_id: &str,
    channel: &str,
    game_root: &Path,
    resource_dir: &Path,
    voice_lang: &str,
    text_lang: &str,
    target_file: &Path,
    source_file: &str,
) -> Result<PathBuf, String>

/// List all backups for a game, sorted by creation time descending.
/// Reads each manifest.json, checks if original file still exists.
pub fn list_backups(game_id: &str) -> Result<Vec<BackupInfo>, String>

/// Read and validate a specific backup manifest.
pub fn read_manifest(backup_path: &Path) -> Result<BackupManifest, String>
```

### Test Requirements

- Unit test: get_backup_dir returns a valid path
- Unit test: create_backup with temp files, verify manifest.json written
- Unit test: list_backups on empty dir returns empty vec
- Unit test: read_manifest with valid and invalid JSON

---

## Module 5: apply_engine

**File**: `core/apply_engine.rs`
**Dependencies**: `std::fs`
**Depends on**: `language_mapper::ApplyPlan`, `backup_manager`, `resource_scanner::compute_sha256`, `logger`

### Types

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub success: bool,
    pub message: String,
    pub backup_path: Option<String>,
    pub rolled_back: bool,
}
```

### Functions

```rust
/// Execute the apply plan:
/// 1. Verify source and target files exist
/// 2. Call backup_manager::create_backup for the target file
/// 3. Copy source to a temp file in the same directory
/// 4. Compute SHA-256 of temp file, compare with source
/// 5. Rename temp file to target (atomic-ish replace)
/// 6. Verify final target SHA-256 matches source
/// 7. Log the operation
///
/// On failure at any step: attempt to restore from backup.
/// On rollback failure: return rolled_back=false with backup_path.
pub fn execute_apply(plan: &ApplyPlan, profile: &GameProfile) -> ApplyResult
```

The function signature requires a `GameProfile` reference so it knows game_root, channel etc. for backup.
Update the command layer accordingly.

### Test Requirements

- Unit test: successful apply with temp files
- Unit test: apply fails if source file missing
- Unit test: apply fails if target file missing
- Unit test: hash verification after copy

---

## Module 6: restore_engine

**File**: `core/restore_engine.rs`
**Dependencies**: `std::fs`
**Depends on**: `backup_manager`, `resource_scanner::compute_sha256`, `logger`

### Types

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub success: bool,
    pub message: String,
}
```

### Functions

```rust
/// Restore from a backup:
/// 1. Read backup manifest
/// 2. Verify backup file integrity (SHA-256)
/// 3. Copy backup original file to temp file in game directory
/// 4. Verify temp file hash
/// 5. Rename temp to target
/// 6. Log the operation
pub fn execute_restore(backup_path: &Path) -> RestoreResult
```

### Test Requirements

- Unit test: successful restore with temp files
- Unit test: restore fails if backup file missing
- Unit test: restore fails if backup manifest invalid

---

## Module 7: logger

**File**: `core/logger.rs`
**Dependencies**: `chrono`, `std::fs`, `dirs`

### Functions

```rust
/// Returns %LOCALAPPDATA%/FHLanguageComboTool/logs/
pub fn get_log_dir() -> Result<PathBuf, String>

/// Append a structured log entry to the daily log file.
/// File: {log_dir}/{date}.log
/// Format: [{ISO8601}] [{level}] [{game}] {message}
pub fn log_operation(game_id: &str, level: &str, message: &str) -> Result<(), String>

/// Read the current day's log file content.
pub fn read_today_log() -> Result<String, String>
```

### Test Requirements

- Unit test: log_operation writes to file
- Unit test: read_today_log returns empty string if no log exists

---

## Module 8: status (command only)

**File**: `commands/status.rs`
**Depends on**: `backup_manager`, `resource_scanner`

### Functions

```rust
/// Check if a game process is running by name.
/// Uses `tasklist` or Windows API via std::process::Command.
pub fn check_game_running(game_id: String) -> Result<bool, String>

/// Determine current configuration state:
/// - "applied" — last backup's target file hash matches current file
/// - "reverted" — target file hash matches original in last backup
/// - "modified" — file exists but hash matches neither
/// - "none" — no backup records exist
pub fn get_status(game_id: String, resource_path: String) -> Result<ConfigStatus, String>
```

---

## Tauri Command Layer

Files in `src-tauri/src/commands/`. These are thin wrappers — no business logic.
Each command calls core module functions and returns the result.

The command layer must be updated to pass `GameProfile` to apply_engine.

---

## Test Coverage Requirements

Each module must have:
1. All public functions covered by at least one unit test
2. At least one error/edge case test per function
3. Tests use `#[cfg(test)]` module with `tempfile` or temp dirs (use `std::env::temp_dir()`)
4. No tests should depend on the real Steam installation or game files
