// src/config_utils.rs
// Manages loading of user-configurable settings like destructive command patterns.

use anyhow::{Context, Result};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const ASK_ME_BEFORE_PATTERNS_FILENAME: &str = "ask_me_before_patterns.txt";

#[cfg(target_os = "windows")]
pub const DEFAULT_ASK_ME_BEFORE_PATTERNS: &[&str] = &[
    "remove-item -recurse",
    "rmdir /s",
    "del /s",
    "format",
    "fdisk",
    "clear-disk",
    "initialize-disk",
    "remove-partition",
    "diskpart",
    // Add more Windows-specific patterns as needed
];

#[cfg(not(target_os = "windows"))]
pub const DEFAULT_ASK_ME_BEFORE_PATTERNS: &[&str] = &[
    "rm -rf",
    "rm -r",
    "mkfs",
    "fdisk",
    "format",
    "dd if=",
    "shred",
    ":(){:|:&};:", // Fork bomb
    "chmod -R 777",
    "mv /* /dev/null",
    // Add more Unix-specific patterns as needed
];

/// Returns the path to the Prime configuration directory (e.g., ~/.prime/).
fn get_prime_config_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))
        .map(|home| home.join(".prime"))
}

/// Loads patterns from a given file in the Prime config directory.
/// If the file doesn't exist or is empty, returns the provided default patterns.
fn load_patterns_from_file(
    config_dir: &Path,
    filename: &str,
    default_patterns: &[&str],
) -> Result<Vec<String>> {
    let file_path = config_dir.join(filename);
    let mut patterns = Vec::new();

    if file_path.exists() {
        let file = fs::File::open(&file_path)
            .with_context(|| format!("Failed to open pattern file: {}", file_path.display()))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line_content = line.with_context(|| {
                format!(
                    "Failed to read line from pattern file: {}",
                    file_path.display()
                )
            })?;
            let trimmed_line = line_content.trim();
            if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
                patterns.push(trimmed_line.to_string());
            }
        }
    }

    if patterns.is_empty() {
        patterns = default_patterns
            .iter()
            .map(|s| s.to_string())
            .collect();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).with_context(|| {
                format!(
                    "Failed to create Prime config directory: {}",
                    config_dir.display()
                )
            })?;
        }
    }
    Ok(patterns)
}

/// Loads "ask me before" (potentially destructive) command patterns (simple string contains).
pub fn load_ask_me_before_patterns() -> Result<Vec<String>> {
    let config_dir = get_prime_config_dir()?;
    load_patterns_from_file(
        &config_dir,
        ASK_ME_BEFORE_PATTERNS_FILENAME,
        DEFAULT_ASK_ME_BEFORE_PATTERNS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_default_ask_me_before_patterns() {
        let temp_config_dir = tempdir().unwrap();
        let patterns = load_patterns_from_file(
            temp_config_dir.path(),
            "test_ask_before.txt",
            DEFAULT_ASK_ME_BEFORE_PATTERNS,
        )
        .unwrap();
        assert_eq!(patterns.len(), DEFAULT_ASK_ME_BEFORE_PATTERNS.len());
    }

    #[test]
    fn test_load_custom_ask_me_before_patterns_from_file() {
        let temp_config_dir = tempdir().unwrap();
        let custom_patterns = ["dangerous-command", "another-risky-operation"];
        fs::write(
            temp_config_dir.path().join("custom_ask_before.txt"),
            custom_patterns.join("\n"),
        )
        .unwrap();

        let patterns = load_patterns_from_file(
            temp_config_dir.path(),
            "custom_ask_before.txt",
            DEFAULT_ASK_ME_BEFORE_PATTERNS,
        )
        .unwrap();
        assert_eq!(patterns.len(), custom_patterns.len());
        assert!(patterns.contains(&"dangerous-command".to_string()));
        assert!(!patterns.contains(&"rm -rf".to_string())); // Default should not be loaded
    }

    #[test]
    fn test_ignore_comments_and_empty_lines() {
        let temp_config_dir = tempdir().unwrap();
        let file_content = r#"
# This is a comment
dangerous-command

# Another comment
risky-operation
"#;
        fs::write(
            temp_config_dir.path().join("test_with_comments.txt"),
            file_content,
        )
        .unwrap();

        let patterns = load_patterns_from_file(
            temp_config_dir.path(),
            "test_with_comments.txt",
            DEFAULT_ASK_ME_BEFORE_PATTERNS,
        )
        .unwrap();
        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&"dangerous-command".to_string()));
        assert!(patterns.contains(&"risky-operation".to_string()));
        assert!(!patterns.iter().any(|p| p.starts_with('#')));
    }
}