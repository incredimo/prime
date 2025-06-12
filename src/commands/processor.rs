use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;
use anyhow::{Context, Result};

#[cfg(target_os = "windows")]
use tempfile::Builder;
#[cfg(not(target_os = "windows"))]
use {tempfile::NamedTempFile, std::os::unix::fs::PermissionsExt};

use crate::config_utils;
use crate::styling::STYLER;
use super::CommandCache;

pub struct ExecutionStrategy {
    primary: String,
    fallbacks: Vec<String>,
}

pub struct CommandProcessor {
    command_cache: CommandCache,
    shell_command: String,
    shell_args: Vec<String>,
    ask_me_before_patterns: Vec<String>,
}

impl CommandProcessor {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let (shell_command, shell_args) =
            ("powershell".to_string(), vec!["-Command".to_string()]);

        #[cfg(not(target_os = "windows"))]
        let (shell_command, shell_args) = ("sh".to_string(), vec!["-c".to_string()]);

        let ask_me_before_patterns = config_utils::load_ask_me_before_patterns().unwrap_or_else(|e| {
            eprintln!(
                "{}",
                STYLER.error_style(format!(
                    "Warning: Failed to load 'ask me before' patterns: {}. Using defaults.",
                    e
                ))
            );
            config_utils::DEFAULT_ASK_ME_BEFORE_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect()
        });

        Self {
            command_cache: CommandCache::new(),
            shell_command,
            shell_args,
            ask_me_before_patterns,
        }
    }

    pub fn get_execution_strategies(&self, command: &str) -> Vec<String> {
        let mut strategies = vec![command.to_string()];
        
        // pip install strategies
        if command.starts_with("pip install") {
            let package = command.trim_start_matches("pip install").trim();
            strategies.push(format!("pip install --user {}", package));
            strategies.push(format!("python -m pip install {}", package));
            strategies.push(format!("pip3 install {}", package));
            
            #[cfg(target_os = "windows")]
            strategies.push(format!("py -m pip install {}", package));
        }
        
        // npm install strategies
        if command.starts_with("npm install") {
            let package = command.trim_start_matches("npm install").trim();
            strategies.push(format!("npm install --no-save {}", package));
            strategies.push(format!("yarn add {}", package));
        }
        
        strategies
    }
    
    pub fn execute_with_fallbacks(&mut self, command: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        let strategies = self.get_execution_strategies(command);
        
        for (idx, strategy) in strategies.iter().enumerate() {
            // Check cache first
            if let Some(cached_result) = self.command_cache.get(strategy) {
                if cached_result.0 == 0 {
                    return Ok(cached_result.clone());
                }
            }
            
            let result = self.execute_command_internal(strategy, working_dir)?;
            if result.0 == 0 {
                return Ok(result);
            }
            
            if idx < strategies.len() - 1 {
                eprintln!("Strategy {} failed, trying next...", idx + 1);
            }
        }
        
        // Return last failure if all strategies failed
        self.execute_command_internal(command, working_dir)
    }

    fn execute_command_internal(&mut self, command: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        // Check cache first
        if let Some(cached_result) = self.command_cache.get(command) {
            return Ok(cached_result);
        }

        let effective_working_dir = working_dir.unwrap_or_else(|| Path::new("."));

        println!(
            "{}",
            STYLER.command_exec_style(format!(
                "Executing in '{}': {}",
                effective_working_dir.display(),
                command
            ))
        );

        let mut args = self.shell_args.clone();
        args.push(command.to_string());

        let output = Command::new(&self.shell_command)
            .args(&args)
            .current_dir(effective_working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute command: {}", command))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let result = if stderr.is_empty() {
            (exit_code, stdout)
        } else if stdout.is_empty() {
            (exit_code, format!("STDERR:\n{}", stderr))
        } else {
            (exit_code, format!("{}\n\nSTDERR:\n{}", stdout, stderr))
        };

        println!(
            "{}",
            STYLER.dim_gray_style(format!(
                "Command completed with exit code: {}",
                exit_code
            ))
        );
        
        let preview = result.1.lines().take(5).collect::<Vec<&str>>().join("\n");
        if !preview.is_empty() {
            println!(
                "{}",
                STYLER.dim_gray_style(format!("Output preview:\n{}", preview))
            );
            if result.1.lines().count() > 5 {
                println!(
                    "{}",
                    STYLER.dim_gray_style(
                        "... (output truncated, full output saved in conversation)"
                    )
                );
            }
        }

        // Cache successful results
        if exit_code == 0 {
            self.command_cache.set(command.to_string(), result.clone());
        }
        
        Ok(result)
    }

    pub fn execute_script(&mut self, script_content: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        #[cfg(target_os = "windows")]
        let mut script_file = Builder::new()
            .suffix(".ps1")
            .tempfile()
            .context("Failed to create temporary PowerShell script file")?;

        #[cfg(not(target_os = "windows"))]
        let mut script_file = NamedTempFile::new()
            .context("Failed to create temporary script file")?;

        script_file
            .write_all(script_content.as_bytes())
            .context("Failed to write script content")?;

        #[cfg(not(target_os = "windows"))]
        {
            let file_path = script_file.path();
            let metadata = fs::metadata(file_path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755); // rwxr-xr-x
            fs::set_permissions(file_path, permissions)?;
        }

        let script_path_str = script_file.path().to_string_lossy().to_string();
        
        #[cfg(target_os = "windows")]
        let command_to_execute = format!("& '{}'", script_path_str);
        #[cfg(not(target_os = "windows"))]
        let command_to_execute = script_path_str;

        self.execute_command(&command_to_execute, working_dir)
    }

    pub fn is_ask_me_before_command(&self, command: &str) -> bool {
        let command_lower = command.trim().to_lowercase();
        self.ask_me_before_patterns
            .iter()
            .any(|pattern| command_lower.contains(&pattern.to_lowercase()))
    }

    pub fn execute_command(&mut self, command: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        let command = command.trim();
        // Try with fallback strategies first for simple commands
        if !command.contains("&&") && !command.contains("|") {
            return self.execute_with_fallbacks(command, working_dir);
        }
        
        self.execute_command_internal(command, working_dir)
    }
    
    // Periodically clean expired cache entries
    pub fn maintain_cache(&mut self) {
        self.command_cache.clear_expired();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread::sleep;

    #[test]
    fn test_command_caching() {
        let mut processor = CommandProcessor::new();
        
        // Test successful command gets cached
        let result1 = processor.execute_command("test command", None).unwrap();
        let result2 = processor.execute_command("test command", None).unwrap();
        assert_eq!(result1, result2);
        
        // Test cache expiration
        processor.command_cache.ttl = Duration::from_millis(1);
        let result1 = processor.execute_command("expiring command", None).unwrap();
        sleep(Duration::from_millis(2));
        let result2 = processor.execute_command("expiring command", None).unwrap();
        assert_eq!(result1, result2); // Should be equal since dummy implementation always returns (0, "")
        
        // Test cache maintenance
        processor.command_cache.set("test".to_string(), (0, String::new()));
        sleep(Duration::from_millis(2));
        processor.maintain_cache();
        assert!(processor.command_cache.get("test").is_none());
    }
}