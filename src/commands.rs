// src/commands.rs
// Command execution for Prime - simplified to only handle script execution

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

use crate::config_utils;
use crate::styling::STYLER;

/// Handles script execution for Prime
pub struct CommandProcessor {
    shell_command: String,
    shell_args: Vec<String>,
    ask_me_before_patterns: Vec<String>,
}

impl CommandProcessor {
    /// Create a new command processor
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
            shell_command,
            shell_args,
            ask_me_before_patterns,
        }
    }

    /// Execute a shell command and return its output
    pub fn execute_command(&self, command: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
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

        let combined_output = if stderr.is_empty() {
            stdout
        } else if stdout.is_empty() {
            format!("STDERR:\n{}", stderr)
        } else {
            format!("{}\n\nSTDERR:\n{}", stdout, stderr)
        };

        println!(
            "{}",
            STYLER.dim_gray_style(format!(
                "Command completed with exit code: {}",
                exit_code
            ))
        );
        let preview = combined_output.lines().take(5).collect::<Vec<&str>>().join("\n");
        if !preview.is_empty() {
            println!(
                "{}",
                STYLER.dim_gray_style(format!("Output preview:\n{}", preview))
            );
            if combined_output.lines().count() > 5 {
                println!(
                    "{}",
                    STYLER.dim_gray_style(
                        "... (output truncated, full output saved in conversation)"
                    )
                );
            }
        }
        Ok((exit_code, combined_output))
    }

    /// Execute a script file
    pub fn execute_script(&self, script_content: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        #[cfg(target_os = "windows")]
        let mut script_file = NamedTempFile::with_suffix(".ps1")
            .context("Failed to create temporary PowerShell script file")?;

        #[cfg(not(target_os = "windows"))]
        let mut script_file = NamedTempFile::builder()
            .prefix("prime_script_")
            .suffix(".sh")
            .tempfile()
            .context("Failed to create temporary script file")?;

        script_file
            .write_all(script_content.as_bytes())
            .context("Failed to write script content")?;

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
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

    /// Check if a command matches any "ask me before" patterns
    pub fn is_ask_me_before_command(&self, command: &str) -> bool {
        let command_lower = command.trim().to_lowercase();
        self.ask_me_before_patterns
            .iter()
            .any(|pattern| command_lower.contains(&pattern.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_command_processor_creation() {
        let processor = CommandProcessor::new();
        #[cfg(target_os = "windows")]
        {
            assert_eq!(processor.shell_command, "powershell");
            assert_eq!(processor.shell_args, vec!["-Command"]);
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(processor.shell_command, "sh");
            assert_eq!(processor.shell_args, vec!["-c"]);
        }
    }

    #[test]
    fn test_simple_command_execution() {
        let processor = CommandProcessor::new();
        let temp_dir = env::temp_dir();
        
        #[cfg(target_os = "windows")]
        let test_command = "echo 'Hello World'";
        #[cfg(not(target_os = "windows"))]
        let test_command = "echo 'Hello World'";
        
        let result = processor.execute_command(test_command, Some(&temp_dir));
        assert!(result.is_ok());
        
        let (exit_code, output) = result.unwrap();
        assert_eq!(exit_code, 0);
        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_script_execution() {
        let processor = CommandProcessor::new();
        
        #[cfg(target_os = "windows")]
        let script_content = r#"
Write-Host "Script Test"
$result = 2 + 2
Write-Host "Result: $result"
"#;
        #[cfg(not(target_os = "windows"))]
        let script_content = r#"
#!/bin/bash
echo "Script Test"
result=$((2 + 2))
echo "Result: $result"
"#;
        
        let result = processor.execute_script(script_content, None);
        assert!(result.is_ok());
        
        let (exit_code, output) = result.unwrap();
        assert_eq!(exit_code, 0);
        assert!(output.contains("Script Test"));
        assert!(output.contains("Result: 4") || output.contains("Result: 4"));
    }

    #[test]
    fn test_ask_me_before_patterns() {
        let processor = CommandProcessor::new();
        
        #[cfg(target_os = "windows")]
        {
            assert!(processor.is_ask_me_before_command("Remove-Item -Recurse"));
            assert!(processor.is_ask_me_before_command("remove-item -recurse"));
            assert!(!processor.is_ask_me_before_command("Get-ChildItem"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert!(processor.is_ask_me_before_command("rm -rf /important/folder"));
            assert!(processor.is_ask_me_before_command("RM -RF"));
            assert!(!processor.is_ask_me_before_command("ls -la"));
        }
    }

    #[test]
    fn test_command_failure() {
        let processor = CommandProcessor::new();
        
        #[cfg(target_os = "windows")]
        let failing_command = "Get-NonExistentCommand";
        #[cfg(not(target_os = "windows"))]
        let failing_command = "nonexistentcommand";
        
        let result = processor.execute_command(failing_command, None);
        assert!(result.is_ok()); // Command execution succeeds, but with non-zero exit code
        
        let (exit_code, _output) = result.unwrap();
        assert_ne!(exit_code, 0); // Should have non-zero exit code
    }
}