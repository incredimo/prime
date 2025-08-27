//! CommandProcessor – safe, cross-platform shell/file primitives for Prime
//! Streamlined UI: this module does no verbose printing;
//! it only prompts for dangerous commands when needed.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use crossterm::style::Stylize;
use glob::Pattern;

use crate::config;
use crate::ui;

const MAX_FILE_READ_LINES: usize = 1000;
const MAX_FILE_READ_BYTES: u64 = 1_048_576; // 1 MB
const MAX_DIR_LISTING_CHILDREN_DISPLAY: usize = 20;
const OUTPUT_PREVIEW_BYTES: usize = 1024;   // 1 KB preview for stdout/stderr

#[inline]
fn looks_binary(buf: &[u8]) -> bool {
    buf.iter().take(256).any(|&b| b == 0)
}

fn human_preview(data: &[u8]) -> String {
    if looks_binary(data) {
        return "[binary data omitted]".to_string();
    }
    let text = String::from_utf8_lossy(data);
    let mut out: String = text.chars().take(OUTPUT_PREVIEW_BYTES).collect();
    if text.len() > OUTPUT_PREVIEW_BYTES {
        out.push_str("\n… (output truncated)");
    }
    out
}

pub struct CommandProcessor {
    shell_command: String,
    shell_args: Vec<String>,
    ignored_path_patterns: Vec<Pattern>,
    ask_me_before_patterns: Vec<String>,
}

impl CommandProcessor {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let (shell_command, shell_args) = ("powershell".to_string(), vec!["-NoLogo".into(), "-Command".into()]);
        #[cfg(not(target_os = "windows"))]
        let (shell_command, shell_args) = ("sh".to_string(), vec!["-c".into()]);

        let ignored_path_patterns = config::load_ignored_path_patterns().unwrap_or_else(|e| {
            eprintln!("{}", format!("Warning: Failed to load ignored path patterns: {}. Using defaults.", e).yellow());
            config::DEFAULT_IGNORED_PATHS
                .iter()
                .filter_map(|s| Pattern::new(s).ok())
                .collect()
        });

        let ask_me_before_patterns = config::load_ask_me_before_patterns().unwrap_or_else(|e| {
            eprintln!("{}", format!("Warning: Failed to load 'ask me before' patterns: {}. Using defaults.", e).yellow());
            config::DEFAULT_ASK_ME_BEFORE_PATTERNS.iter().map(|s| s.to_string()).collect()
        });

        Self { shell_command, shell_args, ignored_path_patterns, ask_me_before_patterns }
    }

    // -------------------------------------------------- //
    // Shell execution
    // -------------------------------------------------- //

    pub fn execute_command(&self, command: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        let _current_dir = working_dir.unwrap_or_else(|| Path::new("."));

        // Ask before dangerous commands (concise prompt).
        for pattern in &self.ask_me_before_patterns {
            if command.contains(pattern) {
                println!("{}", ui::err_tag(&format!("Dangerous command detected (matches '{}')", pattern)));
                print!("Proceed? [y/N]: ");
                std::io::stdout().flush().context("Failed to flush stdout")?;

                let mut line = String::new();
                std::io::stdin().read_line(&mut line).context("Failed to read user input")?;
                if !line.trim().eq_ignore_ascii_case("y") {
                    return Ok((-1, "Command cancelled by user.".into()));
                }
                break;
            }
        }

        // Build argv for chosen shell
        let mut args = self.shell_args.clone();
        args.push(command.to_string());

        let output = Command::new(&self.shell_command)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute command: {}", command))?;

        let exit_code = output.status.code().unwrap_or(-1);

        // Return full merged output; caller decides how to render.
        let mut merged = Vec::new();
        merged.extend_from_slice(&output.stdout);
        if !output.stderr.is_empty() {
            merged.extend_from_slice(b"\n\nSTDERR:\n");
            merged.extend_from_slice(&output.stderr);
        }

        // Optional: quick preview to log internally later if desired.
        let _preview_text = human_preview(&merged);

        Ok((exit_code, String::from_utf8_lossy(&merged).into()))
    }

    pub async fn execute_command_async(&self, command: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        use std::time::Duration;
        
        let _current_dir = working_dir.unwrap_or_else(|| Path::new("."));

        // Ask before dangerous commands (concise prompt).
        for pattern in &self.ask_me_before_patterns {
            if command.contains(pattern) {
                println!("{}", ui::err_tag(&format!("Dangerous command detected (matches '{}')", pattern)));
                print!("Proceed? [y/N]: ");
                std::io::stdout().flush().context("Failed to flush stdout")?;

                let mut line = String::new();
                std::io::stdin().read_line(&mut line).context("Failed to read user input")?;
                if !line.trim().eq_ignore_ascii_case("y") {
                    return Ok((-1, "Command cancelled by user.".into()));
                }
                break;
            }
        }

        // Build argv for chosen shell
        let mut args = self.shell_args.clone();
        args.push(command.to_string());

        let timeout_secs = std::env::var("LLM_SHELL_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);

        let duration = Duration::from_secs(timeout_secs);
        
        match tokio::time::timeout(duration, async {
            tokio::process::Command::new(&self.shell_command)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
        }).await {
            Err(_) => Ok((-1, format!("Command timed out after {:?}", duration))),
            Ok(Ok(output)) => {
                let exit_code = output.status.code().unwrap_or(-1);

                // Return full merged output; caller decides how to render.
                let mut merged = Vec::new();
                merged.extend_from_slice(&output.stdout);
                if !output.stderr.is_empty() {
                    merged.extend_from_slice(b"\n\nSTDERR:\n");
                    merged.extend_from_slice(&output.stderr);
                }

                Ok((exit_code, String::from_utf8_lossy(&merged).into()))
            }
            Ok(Err(e)) => Err(anyhow!("Failed to execute command: {}", e)),
        }
    }

    // -------------------------------------------------- //
    // File helpers
    // -------------------------------------------------- //

    pub fn read_file_to_string_with_limit(&self, path: &Path, line_range: Option<(usize, usize)>) -> Result<(String, bool)> {
        read_file_to_string_with_limit(path, line_range)
    }

    pub fn write_file_to_path(&self, path: &Path, content: &str, append: bool) -> Result<()> {
        write_file(path, content, append)
    }

    pub fn list_directory_smart(&self, path: &Path) -> Result<Vec<String>> {
        list_directory_smart(path, &self.ignored_path_patterns)
    }
}

// ---------------------------------------------------------------------
// Stand-alone utility functions – small & pure for easy unit testing
// ---------------------------------------------------------------------

fn read_file_to_string_with_limit(path: &Path, line_range: Option<(usize, usize)>) -> Result<(String, bool)> {
    let file = fs::File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut truncated = false;
    let content: String;

    if let Some((start, end)) = line_range {
        if start == 0 || start > end {
            return Err(anyhow!("Invalid line range: start must be >= 1 and start <= end. Got start={} end={}", start, end));
        }
        let all_lines: Vec<_> = reader.lines().enumerate().map(|(i, l)| {
            l.with_context(|| format!("Failed to read line {} from file: {}", i + 1, path.display()))
                .unwrap_or_else(|e| {
                    eprintln!("Warning: {}", e);
                    String::new()
                })
        }).collect();
        let total_lines = all_lines.len();

        if start > total_lines {
            content = String::new();
            truncated = end < total_lines;
        } else {
            let effective_end = std::cmp::min(end, total_lines);
            content = all_lines
                .iter()
                .skip(start.saturating_sub(1))
                .take(effective_end - start.saturating_sub(1))
                .cloned()
                .collect::<Vec<_>>()
                .join("\n");
            truncated = end < total_lines;
        }
    } else {
        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_FILE_READ_BYTES {
            let mut limited_reader = BufReader::new(fs::File::open(path).with_context(|| format!("Failed to open file for reading (size limit): {}", path.display()))?).take(MAX_FILE_READ_BYTES);
            let mut buffer = Vec::new();
            limited_reader.read_to_end(&mut buffer).with_context(|| format!("Failed to read file content (size limit): {}", path.display()))?;

            truncated = true;
            if looks_binary(&buffer) {
                content = "[binary data omitted]".into();
            } else {
                let text = String::from_utf8_lossy(&buffer);
                let lines: Vec<&str> = text.lines().take(MAX_FILE_READ_LINES).collect();
                content = lines.join("\n");
            }
        } else {
            let mut tmp = String::new();
            BufReader::new(fs::File::open(path).with_context(|| format!("Failed to open file for reading: {}", path.display()))?).read_to_string(&mut tmp).with_context(|| format!("Failed to read file content: {}", path.display()))?;
            content = tmp;
            truncated = false;
        }
    }

    let mut final_content = content;
    if truncated {
        final_content.push_str("\n… (file content truncated)");
    }

    Ok((final_content, truncated))
}

fn write_file(path: &Path, content: &str, append: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).with_context(|| format!("Failed to create directories for: {}", path.display()))?;
        }
    }

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(append)
        .truncate(!append)
        .open(path)
        .with_context(|| format!("Failed to open file for writing: {}", path.display()))?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write to file: {}", path.display()))
}

fn list_directory_smart(path: &Path, ignored_patterns: &[Pattern]) -> Result<Vec<String>> {
    if !path.is_dir() {
        return Err(anyhow!("Path is not a directory: {}", path.display()));
    }

    let entries = fs::read_dir(path).with_context(|| format!("Failed to read directory: {}", path.display()))?;

    let mut items = Vec::new();
    for entry_result in entries {
        let entry = entry_result.with_context(|| format!("Error reading directory entry in {}", path.display()))?;
        let entry_path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if ignored_patterns.iter().any(|p| p.matches_path(&entry_path) || p.matches(&file_name)) {
            continue;
        }

        let display_name = if entry_path.is_dir() { format!("{}/", file_name) } else { file_name };
        items.push(display_name);
    }

    // Sort: directories first, then case-insensitive alphabetical
    items.sort_by(|a, b| {
        let a_is_dir = a.ends_with('/');
        let b_is_dir = b.ends_with('/');
        if a_is_dir != b_is_dir {
            b_is_dir.cmp(&a_is_dir)
        } else {
            a.to_lowercase().cmp(&b.to_lowercase())
        }
    });

    if items.len() > MAX_DIR_LISTING_CHILDREN_DISPLAY {
        let remaining = items.len() - MAX_DIR_LISTING_CHILDREN_DISPLAY;
        let mut truncated_items = items.into_iter().take(MAX_DIR_LISTING_CHILDREN_DISPLAY).collect::<Vec<_>>();
        truncated_items.push(format!("… (and {} more items)", remaining));
        Ok(truncated_items)
    } else {
        Ok(items)
    }
}
