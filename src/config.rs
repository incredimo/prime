use anyhow::{anyhow, Context, Result};
use crossterm::style::Stylize;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const CONFIG_FILENAME: &str = "config.toml";
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_api_key")]
    pub gemini_api_key: String,
    #[serde(default = "default_api_key")]
    pub ollama_api_key: String,
}

fn default_provider() -> String { "google".to_string() }
fn default_temperature() -> f32 { 0.2 }
fn default_max_tokens() -> u32 { 8192 } // Increased for more complex plans
fn default_api_key() -> String { "".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: None,
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            gemini_api_key: default_api_key(),
            ollama_api_key: default_api_key(),
        }
    }
}

pub fn load_config() -> Result<Config> {
    let config_dir = get_prime_config_dir()?;
    let config_path = config_dir.join(CONFIG_FILENAME);

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create Prime config directory: {}", config_dir.display()))?;
    }

    if !config_path.exists() {
        let default_config = Config::default();
        let toml_string = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;

        let comment = "# Prime Configuration File\n# Set your API keys and preferred models here.\n# You can find your GEMINI_API_KEY at https://aistudio.google.com/app/apikey\n\n";
        let final_content = format!("{}{}", comment, toml_string);

        fs::write(&config_path, final_content)
            .with_context(|| format!("Failed to write default config to {}", config_path.display()))?;

        // Only show the message if API keys are not available in environment
        let has_gemini_key = std::env::var("GEMINI_API_KEY").is_ok();
        let has_ollama_key = std::env::var("OLLAMA_API_KEY").is_ok();

        if !has_gemini_key && !has_ollama_key {
            println!("{}", format!("Configuration file created at {}. Please edit it to add your API keys.", config_path.display()).yellow());
        }
        return Ok(default_config);
    }

    let toml_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file from {}", config_path.display()))?;
    
    let config: Config = toml::from_str(&toml_content)
        .with_context(|| format!("Failed to parse config file at {}", config_path.display()))?;

    Ok(config)
}

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