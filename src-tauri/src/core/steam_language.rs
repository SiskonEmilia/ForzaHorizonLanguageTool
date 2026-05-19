use std::fs;
use std::path::Path;

pub fn code_to_steam_language(code: &str) -> Option<&'static str> {
    match code.to_uppercase().as_str() {
        "EN" | "GB" => Some("english"),
        "JP" => Some("japanese"),
        "CHS" => Some("schinese"),
        "CHT" => Some("tchinese"),
        "FR" => Some("french"),
        "DE" => Some("german"),
        "ES" => Some("spanish"),
        "MX" => Some("latam"),
        "IT" => Some("italian"),
        "PT" => Some("portuguese"),
        "BR" => Some("brazilian"),
        "KO" => Some("korean"),
        "RU" => Some("russian"),
        "PL" => Some("polish"),
        "NL" => Some("dutch"),
        "TR" => Some("turkish"),
        "DK" => Some("danish"),
        "SV" => Some("swedish"),
        "NO" => Some("norwegian"),
        "FI" => Some("finnish"),
        "CZ" => Some("czech"),
        "HU" => Some("hungarian"),
        "EL" => Some("greek"),
        _ => None,
    }
}

pub fn read_manifest_language(manifest_path: &Path) -> Option<String> {
    let content = fs::read_to_string(manifest_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"language\"") {
            return extract_quoted_value(trimmed);
        }
    }
    None
}

pub fn set_manifest_language(manifest_path: &Path, new_language: &str) -> Result<String, String> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read appmanifest: {}", e))?;

    let mut old_language: Option<String> = None;
    let mut output = String::with_capacity(content.len());

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"language\"") {
            if old_language.is_none() {
                old_language = extract_quoted_value(trimmed);
            }
            let indent = &line[..line.len() - line.trim_start().len()];
            output.push_str(&format!("{indent}\"language\"\t\t\"{new_language}\"\n"));
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    fs::write(manifest_path, &output)
        .map_err(|e| format!("Failed to write appmanifest: {}", e))?;

    Ok(old_language.unwrap_or_default())
}

fn extract_quoted_value(line: &str) -> Option<String> {
    let mut in_quotes = false;
    let mut count = 0;
    let mut start = 0;
    for (i, ch) in line.char_indices() {
        if ch == '"' {
            if in_quotes {
                count += 1;
                if count == 2 {
                    return Some(line[start..i].to_string());
                }
                in_quotes = false;
            } else {
                in_quotes = true;
                if count == 1 {
                    start = i + 1;
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("fh_steam_lang_test")
            .join(format!("{}_{}", label, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_code_to_steam_language_known() {
        assert_eq!(code_to_steam_language("EN"), Some("english"));
        assert_eq!(code_to_steam_language("JP"), Some("japanese"));
        assert_eq!(code_to_steam_language("CHS"), Some("schinese"));
        assert_eq!(code_to_steam_language("CHT"), Some("tchinese"));
        assert_eq!(code_to_steam_language("BR"), Some("brazilian"));
        assert_eq!(code_to_steam_language("GB"), Some("english"));
    }

    #[test]
    fn test_code_to_steam_language_case_insensitive() {
        assert_eq!(code_to_steam_language("en"), Some("english"));
        assert_eq!(code_to_steam_language("chs"), Some("schinese"));
    }

    #[test]
    fn test_code_to_steam_language_unknown() {
        assert_eq!(code_to_steam_language("ZZ"), None);
    }

    #[test]
    fn test_set_manifest_language() {
        let dir = temp_dir("set_lang");
        let acf = dir.join("appmanifest_test.acf");
        fs::write(&acf, "\
\"AppState\"
{
\t\"appid\"\t\t\"1551360\"
\t\"name\"\t\t\"Forza Horizon 5\"
\t\"UserConfig\"
\t{
\t\t\"language\"\t\t\"english\"
\t}
\t\"MountedConfig\"
\t{
\t\t\"language\"\t\t\"english\"
\t}
}
").unwrap();

        let old = set_manifest_language(&acf, "japanese").unwrap();
        assert_eq!(old, "english");

        let content = fs::read_to_string(&acf).unwrap();
        assert!(!content.contains("\"english\""));
        assert!(content.contains("\"japanese\""));
        assert!(content.contains("\"appid\""));
        assert!(content.contains("\"Forza Horizon 5\""));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_set_manifest_returns_old_value() {
        let dir = temp_dir("old_val");
        let acf = dir.join("test.acf");
        fs::write(&acf, "\t\"language\"\t\t\"schinese\"\n").unwrap();

        let old = set_manifest_language(&acf, "english").unwrap();
        assert_eq!(old, "schinese");

        let _ = fs::remove_dir_all(&dir);
    }
}
