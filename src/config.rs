use anyhow::{anyhow, Context, Result};
use glob::Pattern;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const IGNORED_PATHS_FILENAME: &str = "ignored_paths.txt";
const ASK_ME_BEFORE_PATTERNS_FILENAME: &str = "ask_me_before_patterns.txt";

pub const DEFAULT_IGNORED_PATHS: &[&str] = &[
    "**/node_modules/**", "**/target/**", "**/.git/**", "**/.hg/**", "**/.svn/**",
    "**/__pycache__/**", "**/.DS_Store", "**/*.pyc", "**/*.swp", "**/.idea/**",
    "**/.vscode/**", "**/build/**", "**/dist/**", "**/.cache/**",
];

#[cfg(target_os = "windows")]
pub const DEFAULT_ASK_ME_BEFORE_PATTERNS: &[&str] = &[
    "remove-item -recurse", "rmdir /s", "del /s", "format", "fdisk", "clear-disk",
    "initialize-disk", "remove-partition", "diskpart",
];

#[cfg(not(target_os = "windows"))]
pub const DEFAULT_ASK_ME_BEFORE_PATTERNS: &[&str] = &[
    "rm -rf", "rm -r", "mkfs", "fdisk", "format", "dd if=", "shred", ":(){:|:&};:",
    "chmod -R 777", "mv /* /dev/null",
];

fn get_prime_config_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not determine home directory"))
        .map(|home| home.join(".prime"))
}

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
                format!("Failed to read line from pattern file: {}", file_path.display())
            })?;
            let trimmed_line = line_content.trim();
            if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
                patterns.push(trimmed_line.to_string());
            }
        }
    }

    if patterns.is_empty() {
        // Use defaults and create the file if it doesn't exist.
        patterns = default_patterns.iter().map(|s| s.to_string()).collect();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).with_context(|| {
                format!("Failed to create Prime config directory: {}", config_dir.display())
            })?;
        }
        let default_content = default_patterns.join("\n");
        fs::write(&file_path, default_content).with_context(|| {
            format!("Failed to write default patterns to {}", file_path.display())
        })?;
    }
    Ok(patterns)
}

pub fn load_ignored_path_patterns() -> Result<Vec<Pattern>> {
    let config_dir = get_prime_config_dir()?;
    let string_patterns =
        load_patterns_from_file(&config_dir, IGNORED_PATHS_FILENAME, DEFAULT_IGNORED_PATHS)?;

    string_patterns
        .iter()
        .map(|s| Pattern::new(s).with_context(|| format!("Invalid glob pattern in ignored paths: {}", s)))
        .collect()
}

pub fn load_ask_me_before_patterns() -> Result<Vec<String>> {
    let config_dir = get_prime_config_dir()?;
    load_patterns_from_file(
        &config_dir,
        ASK_ME_BEFORE_PATTERNS_FILENAME,
        DEFAULT_ASK_ME_BEFORE_PATTERNS,
    )
}