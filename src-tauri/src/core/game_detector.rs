use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const FH5_STEAM_APP_ID: &str = "1551360";
pub const FH6_STEAM_APP_ID: &str = "2483190";
pub const FH5_INSTALL_DIR: &str = "ForzaHorizon5";
pub const FH6_INSTALL_DIR: &str = "ForzaHorizon6";
pub const FH5_EXECUTABLE: &str = "ForzaHorizon5.exe";
pub const FH6_EXECUTABLE: &str = "forzahorizon6.exe";
pub const RESOURCE_SUBPATH: &str = r"media\Stripped\StringTables";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameId {
    Fh5,
    Fh6,
}

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

impl GameId {
    fn steam_app_id(self) -> &'static str {
        match self {
            GameId::Fh5 => FH5_STEAM_APP_ID,
            GameId::Fh6 => FH6_STEAM_APP_ID,
        }
    }

    fn executable(self) -> &'static str {
        match self {
            GameId::Fh5 => FH5_EXECUTABLE,
            GameId::Fh6 => FH6_EXECUTABLE,
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            GameId::Fh5 => "Forza Horizon 5",
            GameId::Fh6 => "Forza Horizon 6",
        }
    }
}

pub fn detect_steam_games() -> Vec<GameProfile> {
    let mut results = Vec::new();

    let steam_path = match read_steam_path() {
        Some(p) => p,
        None => return results,
    };

    let library_paths = match parse_library_folders(&steam_path) {
        Some(paths) => paths,
        None => return results,
    };

    for lib_path in &library_paths {
        for game_id in [GameId::Fh5, GameId::Fh6] {
            let manifest = lib_path
                .join("steamapps")
                .join(format!("appmanifest_{}.acf", game_id.steam_app_id()));

            let content = match std::fs::read_to_string(&manifest) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let install_dir = match vdf_extract_value(&content, "installdir") {
                Some(d) => d,
                None => continue,
            };

            let root = lib_path.join("steamapps").join("common").join(&install_dir);
            if let Ok(profile) = validate_game_directory(&root, game_id) {
                results.push(profile);
            }
        }
    }

    results
}

pub fn validate_game_directory(path: &Path, game_id: GameId) -> Result<GameProfile, String> {
    reject_dangerous_path(path)?;

    let root = path.to_path_buf();

    let exe_path = root.join(game_id.executable());
    if !exe_path.is_file() {
        return Err(format!(
            "Executable '{}' not found in {}",
            game_id.executable(),
            root.display()
        ));
    }

    let resource_path = root.join(RESOURCE_SUBPATH);
    if !resource_path.is_dir() {
        return Err(format!(
            "Resource directory not found: {}",
            resource_path.display()
        ));
    }

    let zip_count = std::fs::read_dir(&resource_path)
        .map_err(|e| format!("Cannot read {}: {e}", resource_path.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        })
        .count();

    if zip_count < 2 {
        return Err(format!(
            "Expected ≥2 .zip files in {}, found {zip_count}",
            resource_path.display()
        ));
    }

    Ok(GameProfile {
        game_id,
        display_name: game_id.display_name().into(),
        channel: "steam".into(),
        steam_app_id: game_id.steam_app_id().into(),
        root_path: root,
        resource_path,
        executable_name: game_id.executable().into(),
    })
}

fn reject_dangerous_path(path: &Path) -> Result<(), String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|_| format!("Path does not exist or is inaccessible: {}", path.display()))?;

    // Reject disk roots (e.g. C:\, D:\)
    if canonical.parent().is_none() {
        return Err(format!("Refusing disk root path: {}", canonical.display()));
    }

    let lower = canonical.to_string_lossy().to_lowercase();

    let windows_dir = std::env::var("SystemRoot")
        .unwrap_or_else(|_| r"C:\Windows".into())
        .to_lowercase();
    if lower.starts_with(&windows_dir) {
        return Err(format!("Refusing system directory: {}", canonical.display()));
    }

    if let Some(home) = dirs::home_dir() {
        let home_lower = home.to_string_lossy().to_lowercase();
        // Reject the home dir itself but allow subdirectories
        if lower == home_lower || lower == format!("{home_lower}\\") {
            return Err(format!("Refusing user home directory: {}", canonical.display()));
        }
    }

    Ok(())
}

fn read_steam_path() -> Option<PathBuf> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let steam_key = hkcu.open_subkey(r"SOFTWARE\Valve\Steam").ok()?;
    let steam_path: String = steam_key.get_value("SteamPath").ok()?;
    let path = PathBuf::from(steam_path);

    if path.is_dir() {
        Some(path)
    } else {
        log::warn!("Steam registry path does not exist: {}", path.display());
        None
    }
}

fn parse_library_folders(steam_path: &Path) -> Option<Vec<PathBuf>> {
    let vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");
    let content = std::fs::read_to_string(&vdf_path)
        .map_err(|e| log::warn!("Cannot read {}: {e}", vdf_path.display()))
        .ok()?;

    Some(vdf_extract_library_paths(&content))
}

// ── Minimal VDF/ACF parser ──────────────────────────────────────────────────

fn vdf_extract_value(content: &str, key: &str) -> Option<String> {
    for token in VdfTokenizer::new(content) {
        if let VdfToken::KeyValue(k, v) = token {
            if k.eq_ignore_ascii_case(key) {
                return Some(v);
            }
        }
    }
    None
}

fn vdf_extract_library_paths(content: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut depth: u32 = 0;
    let mut inside_entry = false;

    for token in VdfTokenizer::new(content) {
        match token {
            VdfToken::BraceOpen => {
                depth += 1;
                // depth 2 = inside a numbered library entry
                if depth == 2 {
                    inside_entry = true;
                }
            }
            VdfToken::BraceClose => {
                if depth == 2 {
                    inside_entry = false;
                }
                depth = depth.saturating_sub(1);
            }
            VdfToken::KeyValue(k, v) if inside_entry && depth == 2 && k == "path" => {
                paths.push(PathBuf::from(v));
            }
            _ => {}
        }
    }

    paths
}

#[derive(Debug)]
enum VdfToken {
    KeyValue(String, String),
    Key(#[allow(dead_code)] String),
    BraceOpen,
    BraceClose,
}

struct VdfTokenizer<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl<'a> VdfTokenizer<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            chars: content.as_bytes(),
            pos: 0,
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.chars.len() {
            let b = self.chars[self.pos];
            if b == b' ' || b == b'\t' || b == b'\r' || b == b'\n' {
                self.pos += 1;
            } else if b == b'/'
                && self.pos + 1 < self.chars.len()
                && self.chars[self.pos + 1] == b'/'
            {
                while self.pos < self.chars.len() && self.chars[self.pos] != b'\n' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn read_quoted_string(&mut self) -> Option<String> {
        if self.pos >= self.chars.len() || self.chars[self.pos] != b'"' {
            return None;
        }
        self.pos += 1; // skip opening quote
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos] != b'"' {
            if self.chars[self.pos] == b'\\' && self.pos + 1 < self.chars.len() {
                self.pos += 1; // skip escaped char
            }
            self.pos += 1;
        }
        let s = String::from_utf8_lossy(&self.chars[start..self.pos]).into_owned();
        if self.pos < self.chars.len() {
            self.pos += 1; // skip closing quote
        }
        // Unescape backslashes (VDF uses \\ for path separators)
        Some(s.replace("\\\\", "\\"))
    }
}

impl Iterator for VdfTokenizer<'_> {
    type Item = VdfToken;

    fn next(&mut self) -> Option<VdfToken> {
        self.skip_whitespace_and_comments();
        if self.pos >= self.chars.len() {
            return None;
        }

        match self.chars[self.pos] {
            b'{' => {
                self.pos += 1;
                Some(VdfToken::BraceOpen)
            }
            b'}' => {
                self.pos += 1;
                Some(VdfToken::BraceClose)
            }
            b'"' => {
                let key = self.read_quoted_string()?;
                self.skip_whitespace_and_comments();
                if self.pos < self.chars.len() && self.chars[self.pos] == b'"' {
                    let value = self.read_quoted_string()?;
                    Some(VdfToken::KeyValue(key, value))
                } else {
                    Some(VdfToken::Key(key))
                }
            }
            _ => {
                self.pos += 1;
                self.next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_LIBRARYFOLDERS: &str = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"apps"
		{
			"228980"		"29803504"
		}
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
		"label"		""
		"apps"
		{
			"1551360"		"84838839"
			"2483190"		"73927492"
		}
	}
}
"#;

    const SAMPLE_APPMANIFEST: &str = r#"
"AppState"
{
	"appid"		"1551360"
	"Universe"		"1"
	"name"		"Forza Horizon 5"
	"StateFlags"		"4"
	"installdir"		"ForzaHorizon5"
	"SizeOnDisk"		"114953550586"
}
"#;

    #[test]
    fn parse_libraryfolders_extracts_paths() {
        let paths = vdf_extract_library_paths(SAMPLE_LIBRARYFOLDERS);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from(r"C:\Program Files (x86)\Steam"));
        assert_eq!(paths[1], PathBuf::from(r"D:\SteamLibrary"));
    }

    #[test]
    fn parse_appmanifest_extracts_installdir() {
        let dir = vdf_extract_value(SAMPLE_APPMANIFEST, "installdir");
        assert_eq!(dir.as_deref(), Some("ForzaHorizon5"));
    }

    #[test]
    fn validate_rejects_empty_temp_dir() {
        let tmp = std::env::temp_dir().join("fh_test_empty_dir");
        let _ = std::fs::create_dir_all(&tmp);
        let result = validate_game_directory(&tmp, GameId::Fh5);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn validate_rejects_disk_root() {
        let result = validate_game_directory(Path::new(r"C:\"), GameId::Fh5);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("root"));
    }

    #[test]
    fn vdf_parser_handles_mixed_whitespace() {
        let input = "\"key1\"\t\t\"value1\"\n\"key2\"  \"value2\"\n\"key3\" \t \"value3\"";
        assert_eq!(vdf_extract_value(input, "key1").as_deref(), Some("value1"));
        assert_eq!(vdf_extract_value(input, "key2").as_deref(), Some("value2"));
        assert_eq!(vdf_extract_value(input, "key3").as_deref(), Some("value3"));
    }
}
