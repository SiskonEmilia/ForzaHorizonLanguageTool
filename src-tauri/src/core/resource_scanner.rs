use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::core::language_mapper;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguagePack {
    pub code: String,
    pub display_name: String,
    pub file_name: String,
    pub path: PathBuf,
    pub size: u64,
    pub sha256: String,
    pub modified_at: String,
    pub readable: bool,
    pub writable: bool,
}

pub fn compute_sha256(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn scan_string_tables(resource_path: &Path) -> Result<Vec<LanguagePack>, String> {
    let entries = fs::read_dir(resource_path).map_err(|e| {
        format!(
            "Failed to read directory {}: {}",
            resource_path.display(),
            e
        )
    })?;

    let mut packs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("zip") {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let code = match path.file_stem().and_then(|s| s.to_str()) {
            Some(stem) => stem.to_string(),
            None => continue,
        };

        let display_name = language_mapper::get_display_name(&code).to_string();

        let metadata = fs::metadata(&path)
            .map_err(|e| format!("Failed to get metadata for {}: {}", path.display(), e))?;

        let size = metadata.len();

        let modified_at = metadata
            .modified()
            .ok()
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt.to_rfc3339()
            })
            .unwrap_or_default();

        let readable = fs::File::open(&path).is_ok();

        let writable = fs::OpenOptions::new().write(true).open(&path).is_ok();

        packs.push(LanguagePack {
            code,
            display_name,
            file_name,
            path,
            size,
            sha256: String::new(),
            modified_at,
            readable,
            writable,
        });
    }

    packs.sort_by(|a, b| a.code.to_lowercase().cmp(&b.code.to_lowercase()));

    Ok(packs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("fh_resource_scanner_test")
            .join(format!("{}_{}", label, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_compute_sha256_known_content() {
        let dir = make_temp_dir("sha256");
        let file = dir.join("test.bin");
        fs::write(&file, b"hello world").unwrap();

        let hash = compute_sha256(&file).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_string_tables_basic() {
        let dir = make_temp_dir("scan_basic");
        fs::write(dir.join("CHS.zip"), b"fake-chs").unwrap();
        fs::write(dir.join("EN.zip"), b"fake-en-data").unwrap();
        fs::write(dir.join("br.zip"), b"fake-br").unwrap();

        let packs = scan_string_tables(&dir).unwrap();
        assert_eq!(packs.len(), 3);

        assert_eq!(packs[0].code, "br");
        assert_eq!(packs[0].display_name, "Português (BR)");
        assert_eq!(packs[0].size, 7);

        assert_eq!(packs[1].code, "CHS");
        assert_eq!(packs[1].display_name, "简体中文");
        assert_eq!(packs[1].size, 8);

        assert_eq!(packs[2].code, "EN");
        assert_eq!(packs[2].display_name, "English");
        assert_eq!(packs[2].size, 12);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_string_tables_empty_dir() {
        let dir = make_temp_dir("scan_empty");

        let packs = scan_string_tables(&dir).unwrap();
        assert!(packs.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_code_preserves_original_case() {
        let dir = make_temp_dir("scan_case");
        fs::write(dir.join("CHS.zip"), b"data").unwrap();
        fs::write(dir.join("br.zip"), b"data").unwrap();
        fs::write(dir.join("En.zip"), b"data").unwrap();

        let packs = scan_string_tables(&dir).unwrap();

        let codes: Vec<&str> = packs.iter().map(|p| p.code.as_str()).collect();
        assert!(codes.contains(&"CHS"));
        assert!(codes.contains(&"br"));
        assert!(codes.contains(&"En"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_results_sorted_case_insensitively() {
        let dir = make_temp_dir("scan_sort");
        fs::write(dir.join("EN.zip"), b"data").unwrap();
        fs::write(dir.join("br.zip"), b"data").unwrap();
        fs::write(dir.join("CHS.zip"), b"data").unwrap();
        fs::write(dir.join("de.zip"), b"data").unwrap();

        let packs = scan_string_tables(&dir).unwrap();
        let codes: Vec<String> = packs.iter().map(|p| p.code.to_lowercase()).collect();
        let mut sorted = codes.clone();
        sorted.sort();
        assert_eq!(codes, sorted);

        let _ = fs::remove_dir_all(&dir);
    }
}
